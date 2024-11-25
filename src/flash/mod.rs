use defmt::info;

mod ipc;
mod thunk;
pub mod monitor;

pub fn init() {
    thunk::init();
}

#[allow(dead_code)]
pub fn apply() -> ! {
    use core::cell::RefCell;
    use embassy_sync::blocking_mutex::Mutex;
    use embassy_boot_rp::{AlignedBuffer, FirmwareUpdaterConfig, BlockingFirmwareUpdater};
    use embassy_rp::flash::Flash;

    let p = unsafe { embassy_rp::Peripherals::steal() };

    const FLASH_SIZE: usize = 2 * 1024 * 1024;

    let flash = Flash::<_, _, FLASH_SIZE>::new_blocking(p.FLASH);
    let flash = Mutex::new(RefCell::new(flash));

    let config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash);

    info!("created FirmwareUpdaterConfig");

    let mut aligned = AlignedBuffer([0; 1]);
    let mut updater = BlockingFirmwareUpdater::new(config, &mut aligned.0);

    // this erases DFU and gives us the writer
    // we don't need this - probe-rs does the erase & write
    //updater.prepare_update();

    info!("marking bootloader state as updated...");
    updater.mark_updated().unwrap(); // sets state parititon, fill to SWAP_MAGIC, i.e. 0xf0

    info!("marked bootloader state as updated");

    // bootloader (already flashed) will now check for 0xf0 (prepare_boot()) and,
    // upon finding all SWAP_MAGICs, indicate it's in State::Swap, do the swap()
    // and boot us. we reset to initiate this:

    info!("resetting...");

    // hack? wait a little bit for defmt
    let mut delay = cortex_m::delay::Delay::new(
        unsafe { cortex_m::Peripherals::steal() }.SYST,
        embassy_rp::clocks::clk_sys_freq(),
    );
    delay.delay_ms(1000);

    cortex_m::peripheral::SCB::sys_reset()
}
