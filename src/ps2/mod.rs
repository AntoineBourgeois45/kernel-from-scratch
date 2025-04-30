pub mod controller;
pub mod mouse;

pub fn init() {
    let _mouse_init_success = mouse::init_mouse();
}