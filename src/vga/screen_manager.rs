use crate::vga::{VGA_WIDTH, VGA_HEIGHT};

const MAX_SCREENS: usize = 2;
const VGA_BUFFER_SIZE: usize = VGA_WIDTH * VGA_HEIGHT;

#[derive(Clone, Copy)]
pub struct ScreenConfig {
	pub name: &'static str,
	pub default_fg_color: u8,
	pub default_bg_color: u8,
	pub show_welcome_message: bool,
}
