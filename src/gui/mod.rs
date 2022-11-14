mod cpu_state;
mod memory_view;
mod menu;

use crate::cpu::Cpu;
use cpu_state::CpuState;
use eframe::egui;
use egui::Sense;
use memory_view::MemoryView;
use menu::MenuBar;

#[derive(Default)]
pub struct App {
	cpu: Cpu,
	menu_bar: MenuBar,
	cpu_state: CpuState,
	memory_view: MemoryView,
	run: bool,
}

impl App {
	pub fn new(cpu: Cpu) -> Self {
		Self {
			cpu,
			..Default::default()
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		if self.run {
			self.cpu.step();
		}

		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			self.menu_bar.ui(ui, frame);
		});

		egui::TopBottomPanel::top("toolbar_panel").show(ctx, |ui| {
			ui.horizontal_centered(|ui| {
				if self.run {
					if ui.button("Pause").clicked() {
						self.run = false;
					}
				} else if ui.button("Run").clicked() {
					self.run = true;
				}

				let sense_type = if self.run {
					Sense::hover()
				} else {
					Sense::click()
				};

				if ui
					.add(egui::Button::new("Step").sense(sense_type))
					.clicked()
				{
					self.cpu.step();
				}

				if ui.button("Reset").clicked() {
					self.cpu.reset();
				}
			});
		});

		egui::SidePanel::right("right_panel")
			.resizable(false)
			.show(ctx, |ui| {
				self.cpu_state.ui(ui, &self.cpu);
			});

		egui::TopBottomPanel::bottom("bottom_panel")
			.min_height(200.0)
			.resizable(false)
			.show(ctx, |ui| {
				self.memory_view.ui(ui, &mut self.cpu);
			});

		egui::CentralPanel::default().show(ctx, |ui| {
			egui::warn_if_debug_build(ui);
		});
		ctx.request_repaint();
	}
}
