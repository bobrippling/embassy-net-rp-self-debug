use core::sync::atomic::Ordering;
use defmt::{info, error};
use embassy_rp::rom_data;

use crate::flash::{
    ipc::{IPC, IpcWhat},
    thunk::Operation,
};

pub enum FlashMessage {
    Operation,
    Complete,
}

pub fn handle_pending_flash() -> Option<FlashMessage> {
    #[allow(static_mut_refs)]
    let ipc = unsafe { &IPC };

    let msg = match ipc.read_what() {
        Ok(None) => return None,

        Ok(Some(IpcWhat::Init)) => {
            info!(
                "found init({:#x}, {:#x}, {:#x}), ignoring",
                ipc.regs[0],
                ipc.regs[1],
                ipc.regs[2],
            );

            FlashMessage::Operation
        }
        Ok(Some(IpcWhat::Deinit)) => {
            info!(
                "found deinit({:#x}), ignoring",
                ipc.regs[0],
            );

            if ipc.regs[0] == Operation::Program as usize {
                info!("deinit(Operation::Program) detected, finalising...");

                FlashMessage::Complete
            } else {
                FlashMessage::Operation
            }
        }
        Ok(Some(IpcWhat::Program)) => {
            info!(
                "found program_page({:#x}, {:#x}, {:#x}), programming...",
                ipc.regs[0],
                ipc.regs[1],
                ipc.regs[2],
            );


            flash_safe(|| {
                // count and data are passed reversed, see probe-rs:
                // 0eaed1a2461ca, src/flashing/flasher.rs, L849-L851
                let [addr, count, data] = ipc.regs;

                let addr = flash_map_address(addr as u32);
                let count = count as usize;
                let data = data as *const u8;

                debug_assert!(
                    addr as usize % embassy_rp::flash::WRITE_SIZE == 0,
                    "buffers must be aligned"
                ); // trivial

                unsafe {
                    // SAFETY (TODO):
                    // - interrupts disabled
                    // - 2nd core is running code in ram (flash algo), interrupts also disabled
                    // - DMA is not accessing flash
                    rom_data::flash_range_program(addr, data, count) // "RP"
                }
            });

            info!("program_page done");
            FlashMessage::Operation
        }
        Ok(Some(IpcWhat::Erase)) => {
            info!(
                "found erase_sector({:#x}), erasing...",
                ipc.regs[0],
            );

            flash_safe(|| {
                let addr = flash_map_address(ipc.regs[0] as u32);
                let (count, block_size, block_cmd) = (0x1000, 0x10000, 0xd8);

                unsafe {
                    // SAFETY:
                    // - interrupts disabled
                    // - 2nd core is running code in ram (flash algo), interrupts also disabled
                    // - DMA is not accessing flash
                    rom_data::flash_range_erase(addr, count, block_size, block_cmd) // "RE"
                }
            });

            info!("erase done");
            FlashMessage::Operation
        }
        Err(v) => {
            error!("unknown ipc value {}", v);
            FlashMessage::Operation
        }
    };

    ipc.what.store(0, Ordering::SeqCst);

    Some(msg)
}

fn flash_map_address(addr: u32) -> u32 {
    extern "C" {
        static __bootloader_active_start: u32;
        static __bootloader_dfu_start: u32;
    }

    // 1. Addresses are given to us relative to memory, we want them relative to flash.
    //    Flash is mapped at 0x10000000, so we subtract that
    // 2. Addresses are for FLASH (memory.x), we want to write into DFU,
    // so add the offset from FLASH to DFU

    let active_start = unsafe { &__bootloader_active_start as *const _ as u32 };
    let dfu_start = unsafe { &__bootloader_dfu_start as *const _ as u32 };
    let dfu_offset = dfu_start - active_start;

    addr - 0x10000000 + dfu_offset
}

fn flash_safe(cb: impl FnOnce()) {
    use embassy_rp::pac as pac;

    assert!(pac::SIO.cpuid().read() == 0, "must be on core0");

    // init
    unsafe {
        // SAFETY:
        // none known
        rom_data::connect_internal_flash(); // "IF"
        rom_data::flash_exit_xip(); // "EX"
    }

    cortex_m::interrupt::free(|_| {
        // TODO: wait for dma to finish

        cb()
    });

    // deinit
    unsafe {
        // SAFETY (TODO):
        // none known
        rom_data::flash_flush_cache(); // "FX"
        rom_data::flash_enter_cmd_xip(); // "CX"
    }
}
