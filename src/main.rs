#![forbid(unsafe_code)]

#[cfg(test)]
mod tests;

mod cpu;
mod gui;
mod memory;
mod system;
pub mod utils;

use cpu::Cpu;
use gui::App;

fn main() {
	let options = eframe::NativeOptions {
		initial_window_size: Some(egui::vec2(1400.0, 900.0)),
		min_window_size: Some(egui::vec2(1400.0, 900.0)),
		default_theme: eframe::Theme::Dark,
		..Default::default()
	};

	eframe::run_native(
		"ATmega328p Emulator",
		options,
		Box::new(|_cc| Box::new(App::new(Cpu::init()))),
	);
}
