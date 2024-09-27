#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU8, Ordering};
use core::ptr::NonNull;

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

// 3 * size_of::<usize>() + usize = 4 words = 16 bytes

#[repr(C)]
enum IpcWhat {
    Initialised = 1, // anything but zero
    Deinitalised,
    Programming,
    Erasing,
}

#[repr(C)]
struct Ipc {
    what: AtomicU8, // IpcWhat,
    regs: [usize; 3],
}

// from RP2040.yaml, we should live in the given memory range along with the block of memory to flash,
// but not our stack, which lives in core1 stack's zone
// FIXME: no hardcoding
static mut IPC: Option<NonNull<Ipc>> = None;

#[link_section = ".text"]
fn ipc(what: IpcWhat, regs: &[usize; 3]) {
    let ipc = unsafe { get_ipc() };

    ipc.regs.copy_from_slice(regs);
    ipc.what.store(what as u8, Ordering::SeqCst);
}

#[link_section = ".text"]
fn ipc_wait() -> ! {
    let ipc: &Ipc = unsafe { get_ipc() };

    while ipc.what.load(Ordering::Relaxed) > 0 {
    }

    let exit_code = 0;
    halt(exit_code)
}

#[link_section = ".text"]
fn halt(exit_code: usize) -> ! {
    unsafe {
        core::arch::asm!(
            "1: wfi\nb 1b",
            in("r0") exit_code,
            options(noreturn, nomem, nostack, preserves_flags)
        );
    }
}

#[link_section = ".text"]
#[no_mangle]
fn init(address: usize, clock_or_zero: usize, op: Operation) -> ! {
    ipc(
        IpcWhat::Initialised,
        &[address, clock_or_zero, op as _]
    );

    ipc_wait()
}

#[link_section = ".text"]
#[no_mangle]
fn uninit(op: Operation) -> ! {
    ipc(
        IpcWhat::Deinitalised,
        &[op as _, 0, 0]
    );

    ipc_wait()
}

#[link_section = ".text"]
#[no_mangle]
fn program_page(address: usize, byte_len: usize, buffer: *const u8) -> ! {
    ipc(
        IpcWhat::Programming,
        &[address, byte_len, buffer as _]
    );

    ipc_wait()
}

#[link_section = ".text"]
#[no_mangle]
fn erase_sector(address: usize) -> ! {
    ipc(
        IpcWhat::Erasing,
        &[address, 0, 0]
    );

    ipc_wait()
}

unsafe fn get_ipc() -> &'static mut Ipc {
    unsafe {
        IPC.get_or_insert_with(scan_mem_for_ipc).as_mut()
    }
}

fn scan_mem_for_ipc() -> NonNull<Ipc> {
    let start = 0x20000000;
    let end = 0x20042000;

    let mut target_bytes: [u8; 16] = [0; 16];

    target_bytes[0] = b'S';
    target_bytes[1] = b'E';
    target_bytes[2] = b'L';
    target_bytes[3] = b'F';
    target_bytes[4] = b'D';
    target_bytes[5] = b'B';
    target_bytes[6] = b'G';
    target_bytes[7] = b'_';
    target_bytes[8] = b'S';
    target_bytes[9] = b'I';
    target_bytes[10] = b'G';
    target_bytes[11] = b'_';
    target_bytes[12] = b'7';
    target_bytes[13] = b'4';
    target_bytes[14] = b'9';

    return unsafe {
        NonNull::new_unchecked(find(start, end, &target_bytes) as _)
    };

    fn find(start: usize, end: usize, target: &[u8]) -> usize {
        for ptr in start..end {
            let candidate = unsafe {
                core::slice::from_raw_parts(ptr as *const u8, target.len())
            };

            if candidate == target { // FIXME: calls memcmp()
                return ptr;
            }
        }

        loop {} // panic
    }
}

/*
// necessary to link / a hack:
use defmt_rtt as _;

embassy_rp::bind_interrupts!(struct Irqs {

});

#[cortex_m_rt::entry]
fn main() -> ! {
    loop {}
}

#[panic_handler]
fn on_panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
*/
