mod ipc;
pub mod thunk;
pub mod monitor;

pub fn init() {
    thunk::init();
}
