#[derive(Default)]
pub struct MenuBar {}

impl MenuBar {
	pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
		egui::menu::bar(ui, |ui| {
			egui::widgets::global_dark_light_mode_switch(ui);
			ui.separator();

			if ui.button("Import file").clicked() {}

			if ui.button("Quit").clicked() {
				frame.close();
			}
		});
	}
}
