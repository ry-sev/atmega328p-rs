#![forbid(unsafe_code)]

#[cfg(test)]
mod tests;

mod cpu;
mod gui;
mod memory;
mod system;
pub mod utils;

use cpu::Cpu;
use eframe::Theme;
use gui::App;
use system::System;

fn main() {
	let options = eframe::NativeOptions {
		initial_window_size: Some(egui::vec2(1400.0, 900.0)),
		min_window_size: Some(egui::vec2(1400.0, 900.0)),
		default_theme: Theme::Light,
		..Default::default()
	};

	eframe::run_native(
		"ATmega328p Emulator",
		options,
		Box::new(|_cc| Box::new(App::new(Cpu::init(System::default())))),
	);
}
