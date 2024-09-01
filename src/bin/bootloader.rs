#![no_std]
#![no_main]

use defmt::info;
use {defmt_rtt as _, panic_probe as _};

// static mut CORE1_STACK: Stack<4096> = Stack::new();
// static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
// static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

//#[link_section = ".boot3.data"]
static mut _BOOT_JOURNAL: u8 = 0;

//#[link_section = ".boot3.data"]
static mut _BOOT_STATE: u8 = 0;

//#[link_section = ".boot3.text"]
#[cortex_m_rt::entry]
fn boot() -> ! {
    extern "C" {
        static _stack_start: u8;
        static _user_main: u8;
    }

    info!("bootloader boot()");

    unsafe {
        assert!(_BOOT_JOURNAL == _BOOT_STATE, "unflashed or interrupt flash, refusing to boot");

        // we now have a valid section of user code, jump to it

        // from embassy-boot-rp
        // TODO
        // #[allow(unused_mut)]
        // let mut p = cortex_m::Peripherals::steal();
        // #[cfg(not(armv6m))]
        // p.SCB.invalidate_icache();
        // p.SCB.vtor.write(start);

        let stack = Symbol(&_stack_start);
        let user_code = Symbol(&_user_main);

        cortex_m::asm::bootload([
            user_code.as_u32(),
            stack.as_u32(),
        ].as_ptr())
    }
}

#[derive(Copy, Clone)]
struct Symbol(*const u8);

impl From<&u8> for Symbol {
    //#[link_section = ".boot3.data"]
    fn from(value: &u8) -> Self {
        Self(value as _)
    }
}

impl Symbol {
    //#[link_section = ".boot3.data"]
    fn as_u32(self) -> u32 {
        self.0 as _
    }
}

/*
mod dap;
mod dap_leds;
mod jtag;
mod network;
mod swd;
mod swj;
mod swo;

use cortex_m::asm::nop;
use cyw43_pio::PioSpi;
use dap::dap::DapVersion;
use dap_leds::DapLeds;
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_net::tcp::TcpSocket;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::pac::SYSCFG;
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::{bind_interrupts, clocks};
use embassy_time::Duration;
use embedded_io_async::Write;
use static_cell::StaticCell;
use swj::Swj;
use swo::Swo;

// bind_interrupts!(struct Irqs0 {
//     PIO0_IRQ_0 => InterruptHandler<PIO0>;
// });

#[embassy_executor::main]
async fn main(#[allow(unused_variables)] spawner: Spawner) {
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(core1_task())));
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

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_task(spawner, spi, p.PIN_23))))
}

#[embassy_executor::task]
async fn core0_task(
    spawner: Spawner,
    spi: PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    pin_23: PIN_23,
) {
    let stack = network::init_network(
        spawner,
        network::Mode::Station,
        env!("WIFI_SSID"),
        env!("WIFI_PASSPHRASE"),
        network::Address::Dhcp,
        spi,
        Output::new(pin_23, Level::Low),
    )
    .await;

    let mut rx_buffer = [0; dap::dap::DAP2_PACKET_SIZE as usize];
    let mut tx_buffer = [0; dap::dap::DAP2_PACKET_SIZE as usize];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(30)));

    let swj = Swj::new(swd::Swd::new(clocks::clk_sys_freq(), SYSCFG.dbgforce()));
    let mut dap = dap::dap::Dap::new(swj, DapLeds::new(), Swo::new(), "VERSION");

    loop {
        info!("Waiting for connection");
        if socket.accept(1234).await.is_err() {
            warn!("Failed to accept connection");
            continue;
        }

        info!("Connected");

        loop {
            let mut request_buffer = [0; dap::dap::DAP2_PACKET_SIZE as usize];

            info!("Waiting for request");

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

            info!("Received {} bytes", n);

            let mut response_buffer = [0; dap::dap::DAP2_PACKET_SIZE as usize];
            let n = dap
                .process_command(&request_buffer[..n], &mut response_buffer, DapVersion::V2)
                .await;

            info!("Responding with {} bytes", n);

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
*/
