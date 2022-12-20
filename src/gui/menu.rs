use std::path::PathBuf;

use crate::system::System;

fn find_hex_files() -> glob::Paths {
	let exe_path = std::env::current_exe();
	let programs_path = exe_path.unwrap().parent().unwrap().join("../../programs");
	glob::glob(programs_path.join("**/*.hex").to_str().unwrap()).unwrap()
}

pub struct MenuBar {
	programs: Vec<PathBuf>,
}

impl Default for MenuBar {
	fn default() -> Self {
		let programs = find_hex_files().map(|res| res.unwrap()).collect();
		Self { programs }
	}
}

impl MenuBar {
	pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, system: &mut System) {
		egui::menu::bar(ui, |ui| {
			egui::widgets::global_dark_light_mode_switch(ui);
			ui.separator();

			ui.menu_button("Import", |ui| {
				for program_file in &self.programs {
					let filename = program_file.file_name().unwrap().to_str().unwrap();
					if ui.button(filename).clicked() {
						system.flash_from_hex_file(program_file);
					}
				}
			});

			if ui.button("Quit").clicked() {
				frame.close();
			}
		});
	}
}
