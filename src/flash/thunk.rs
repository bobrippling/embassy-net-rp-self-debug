#![allow(static_mut_refs)]

use core::{arch::asm, mem::size_of, sync::atomic::Ordering};
use defmt::info;

use super::ipc::IpcWhat;

type Thunk = [extern "C" fn(usize, usize, usize) -> usize; 4];

static ALGO_THUNK: Thunk =
    [on_init, uninit, program_page, erase_sector];

#[allow(dead_code)]
#[repr(C)]
pub enum Operation {
    Erase = 1,
    Program = 2,
    Verify = 3,
}

// #[repr(C)]
// enum IpcWhat {
//     Initialised {
//         address: *const u8,
//         clock_or_zero: usize,
//         op: Operation,
//     },
//     Deinitalised {
//         op: Operation,
//     },
//     Programming {
//         addr: *const u8,
//         byte_len: usize,
//         buffer: *const u8,
//     },
//     Erasing {
//         addr: *const u8,
//     },
// }

fn thunk_ptr() -> *mut Thunk {
    let size = size_of::<extern "C" fn(usize, usize, usize) -> usize>();
    let base_address: usize = 0x21040000 - size * ALGO_THUNK.len();
    base_address as _
}

pub fn init() {
    // TODO: Convert this to linker magic
    let base_address = thunk_ptr();
    info!("writing thunk to {:#x}", base_address);

    unsafe {
        core::ptr::write(base_address, ALGO_THUNK)
    }
}

extern "C" fn on_init(
    address: usize,
    clock_or_zero: usize,
    op: usize, /* Operation */
) -> usize {
    info!(
        "flash algo, executing on_init(address={:#x}, clk_or_zero={}, op={})",
        address, clock_or_zero, op
    );
    ipc(IpcWhat::Init, &[address, clock_or_zero, op as _]);
    info!("flash algo, posted IPC, waiting...");

    ipc_wait()
}

extern "C" fn uninit(op: usize /*Operation*/, _: usize, _: usize) -> usize {
    info!("flash algo, executing uninit(op={})", op);
    ipc(IpcWhat::Deinit, &[op as _, 0, 0]);

    ipc_wait()
}

extern "C" fn program_page(address: usize, byte_len: usize, buffer: usize) -> usize {
    info!(
        "flash algo, executing program_page(address={:#x}, byte_len={}, buffer={:#x})",
        address, byte_len, buffer,
    );
    let buffer = buffer as *const u8;

    ipc(IpcWhat::Program, &[address, byte_len, buffer as _]);

    ipc_wait()
}

fn raise_to_probe_rs(x: i32) {
    unsafe {
        asm!(
            "bkpt #0000",
            in("r0") x,
        );
    }
}

extern "C" fn erase_sector(address: usize, _: usize, _: usize) -> usize {
    info!("flash algo, executing erase_sector(address={:#x})", address);

    info!("sp = {}", {
        let sp = unsafe {
            let mut sp: *const ();
            asm!("mov {x}, sp", x = out(reg) sp);
            sp
        };
        sp
    });

    info!("ipc: {:?}", {
        let r = unsafe { &super::ipc::IPC };
        (
            r.what.load(Ordering::Relaxed),
            r.regs,
        )
    });

    let mut delay = cortex_m::delay::Delay::new(
        unsafe { cortex_m::Peripherals::steal() }.SYST,
        embassy_rp::clocks::clk_sys_freq(),
    );
    delay.delay_ms(10000);

    // raise_to_probe_rs(1234);
    //ipc(IpcWhat::Erase, &[address, 0, 0]); // problem is here
    {
        let ipc = unsafe { &mut super::ipc::IPC };

        ipc.regs.copy_from_slice(&[address, 0, 0]);
        ipc.what.store(IpcWhat::Erase as u8, Ordering::SeqCst);
    }
    raise_to_probe_rs(5678);

    ipc_wait()
}

fn ipc(what: IpcWhat, regs: &[usize; 3]) {
    let ipc = unsafe { &mut super::ipc::IPC };

    ipc.regs.copy_from_slice(regs); // FIXME: could use a &[usize] here / in callers
    ipc.what.store(what as u8, Ordering::SeqCst); // FIXME: Release
}

fn ipc_wait() -> usize {
    let ipc = unsafe { &super::ipc::IPC };

    cortex_m::interrupt::free(|_| while ipc.what.load(Ordering::Relaxed) > 0 {});

    info!("flash algo, got fin, exiting...");

    info!("thunk: {:?}", unsafe { *thunk_ptr() });

    0
}
