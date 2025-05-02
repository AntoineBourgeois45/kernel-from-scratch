pub mod controller;
pub mod mouse;
pub mod keyboard;

pub fn init() {
    let _mouse_init_success = mouse::init_mouse();
}