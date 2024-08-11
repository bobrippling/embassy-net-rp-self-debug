#[link_section = ".boot3.data"]
static mut _BOOT_JOURNAL: u8 = 0;

#[link_section = ".boot3.data"]
static mut _BOOT_STATE: u8 = 1;

#[link_section = ".boot3.data"]
pub fn boot() -> ! {
    extern "C" {
        static _stack_start: u8;
    }
    let stack = Symbol(unsafe { &_stack_start });

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
        let user_code = Symbol(&crate::user_main as *const _ as *const _);

        cortex_m::asm::bootload([
            user_code.as_u32(),
            stack.as_u32(),
        ].as_ptr())
    }
}

#[repr(C)]
enum BootState {
    A,
    B,
}

impl TryFrom<u8> for BootState {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::A,
            1 => Self::A,
            _ => return Err(())
        })
    }
}

#[derive(Copy, Clone)]
struct Symbol(*const u8);

impl From<&u8> for Symbol {
    fn from(value: &u8) -> Self {
        Self(value as _)
    }
}

impl Symbol {
    fn as_u32(self) -> u32 {
        self.0 as _
    }
}
