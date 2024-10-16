#![no_std]
#![no_main]

mod dap;
mod dap_leds;
mod jtag;
mod network;
mod swd;
mod swj;
mod swo;
mod flash;

use cyw43_pio::PioSpi;
use dap::dap::DapVersion;
use dap_leds::DapLeds;
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::pac::SYSCFG;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::{bind_interrupts, clocks};
use embassy_time::Duration;
use embedded_io_async::Write;
use swj::Swj;
use swo::Swo;

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs0 {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

static mut CORE1_STACK: Stack<4096> = Stack::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");
    flash::init();

    let p = embassy_rp::init(Default::default());

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            loop {
                cortex_m::asm::nop();
            }
        },
    );

    let mut pio = Pio::new(p.PIO0, Irqs0);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        Output::new(p.PIN_25, Level::High),
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    info!("init'ing network...");
    let stack = network::init_network(
        spawner,
        network::Mode::Station,
        env!("WIFI_SSID"),
        env!("WIFI_PASSPHRASE"),
        network::Address::Dhcp,
        spi,
        Output::new(p.PIN_23, Level::Low),
    )
    .await;

    info!("network ready");

    let mut rx_buffer = [0; dap::dap::DAP2_PACKET_SIZE as usize];
    let mut tx_buffer = [0; dap::dap::DAP2_PACKET_SIZE as usize];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(30)));
    info!("socket setup");

    let swj = Swj::new(swd::Swd::new(clocks::clk_sys_freq(), SYSCFG.dbgforce()));
    let mut dap = dap::dap::Dap::new(swj, DapLeds::new(), Swo::new(), "VERSION");
    info!("dap setup");

    loop {
        info!("Waiting for connection");
        if socket.accept(1234).await.is_err() {
            warn!("Failed to accept connection");
            continue;
        }

        info!("Connected");

        loop {
            let mut request_buffer = [0; dap::dap::DAP2_PACKET_SIZE as usize];

            trace!("Waiting for request");

            let n = match socket.read(&mut request_buffer).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("read error: {:?}", e);
                    break;
                }
            };

            trace!("Received {} bytes", n);

            let mut response_buffer = [0; dap::dap::DAP2_PACKET_SIZE as usize];
            let n = dap
                .process_command(&request_buffer[..n], &mut response_buffer, DapVersion::V2)
                .await;

            // possibly move this to a polling task
            // or just use proper IPC / SIO.fifo
            flash::monitor::handle_pending_flash();

            trace!("Responding with {} bytes", n);

            match socket.write_all(&response_buffer[..n]).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("write error: {:?}", e);
                    break;
                }
            };
        }

        dap.suspend();
        socket.abort();
        let _ = socket.flush().await;
    }
}
