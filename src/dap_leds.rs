use crate::dap::dap;

pub struct DapLeds();

impl DapLeds {
    pub fn new() -> Self {
        Self()
    }
}

impl dap::DapLeds for DapLeds {
    fn react_to_host_status(&mut self, _host_status: dap::HostStatus) {
        // defmt::info!("Host status: {:?}", host_status);
    }
}
