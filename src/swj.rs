use defmt::{debug, trace, warn};
use embassy_time::{Duration, Ticker};

use crate::{dap, jtag::Jtag, swd::Swd};

pub struct Swj {
    pub(super) swd: Swd,
}

impl Swj {
    pub fn new(swd: Swd) -> Self {
        Self { swd }
    }
}

impl dap::swj::Dependencies<Swd, Jtag> for Swj {
    fn process_swj_pins(
        &mut self,
        _output: crate::dap::swj::Pins,
        _mask: crate::dap::swj::Pins,
        _wait_us: u32,
    ) -> crate::dap::swj::Pins {
        todo!()
    }

    async fn process_swj_sequence(&mut self, data: &[u8], mut bits: usize) {
        let was_attached = self.swd.dbgforce.modify(|r| {
            let was = r.proc1_attach();
            r.set_proc1_attach(true);
            was
        });

        if !was_attached {
            // initial attach, do a sequence
            const CORE_SEL: u8 = 0x22; // 0x00 for core0, 0x22 for core1
            debug!("initial attach, swj sequence");
            self.process_swj_sequence(
                &[0x99, 0xff, 0x24, 0x05, 0x20, CORE_SEL, 0x00],
                7 * 8
            );
            debug!("initial attach, swj sequence done");
        }

        let mut ticker = Ticker::every(Duration::from_ticks(self.swd.half_period_ticks as u64));
        ticker.next().await;

        trace!("Running SWJ sequence: {:08b}, len = {}", data, bits);
        for byte in data {
            let mut byte = *byte;
            let frame_bits = core::cmp::min(bits, 8);
            for _ in 0..frame_bits {
                let bit = byte & 1;
                byte >>= 1;
                if bit != 0 {
                    self.swd.dbgforce.modify(|r| r.set_proc1_swdi(true));
                } else {
                    self.swd.dbgforce.modify(|r| r.set_proc1_swdi(false));
                }
                self.swd.dbgforce.modify(|r| r.set_proc1_swclk(false));
                ticker.next().await;
                self.swd.dbgforce.modify(|r| r.set_proc1_swclk(true));
                ticker.next().await;
            }
            bits -= frame_bits;
        }
    }

    fn process_swj_clock(&mut self, _max_frequency: u32) -> bool {
        todo!()
    }

    fn high_impedance_mode(&mut self) {
        if true {
            self.swd.dbgforce.modify(|r| r.set_proc1_attach(false));
        } else {
            warn!("high impedance mode, untested");
            self.swd.dbgforce.modify(|r| {
                // swdio low
                r.set_proc1_swdi(false);
                r.set_proc1_swdo(false);
            });
            self.swd.dbgforce.modify(|r| {
                r.set_proc1_swclk(false);
            });
            // nreset floating disabled - jtag only?
        }
    }
}

impl From<Swd> for Swj {
    fn from(swd: Swd) -> Self {
        Self { swd }
    }
}

impl From<Jtag> for Swj {
    fn from(_: Jtag) -> Self {
        todo!()
    }
}
