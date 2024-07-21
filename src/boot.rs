pub fn boot() -> ! {
    static mut _BOOT_STATE: u8 = BootState::A as _;

    extern "C" {
        static _stack_start: u8;
        static _boot_ab_a: u8;
        static _boot_ab_b: u8;
    }
    let stack = Symbol(unsafe { &_stack_start });

    let boot_state: BootState = unsafe { _BOOT_STATE }
        .try_into()
        .expect("invalid boot state");

    let reset_code = Symbol(match boot_state {
        BootState::A => unsafe { &_boot_ab_a }
        BootState::B => unsafe { &_boot_ab_b }
    });

    unsafe {
        // from embassy-boot-rp
        // TODO
        // #[allow(unused_mut)]
        // let mut p = cortex_m::Peripherals::steal();
        // #[cfg(not(armv6m))]
        // p.SCB.invalidate_icache();
        // p.SCB.vtor.write(start);

        cortex_m::asm::bootload([
            reset_code.as_u32(),
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
