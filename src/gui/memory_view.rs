use crate::{cpu::Cpu, memory::Memory};
use std::ops::Range;

const PADDING_SIZE: f32 = 4.0;

#[derive(PartialEq, Eq)]
enum Tab {
	ProgramFlash,
	DataMemory,
	Eeprom,
}

struct MemoryTab {
	column_count: usize,
}

impl Default for MemoryTab {
	fn default() -> Self {
		Self { column_count: 16 }
	}
}

impl MemoryTab {
	fn ui(&mut self, ui: &mut egui::Ui, memory: &mut impl Memory) {
		let scroll = egui::ScrollArea::vertical()
			.max_height(f32::INFINITY)
			.auto_shrink([false; 2]);

		let text_style = egui::TextStyle::Body;
		let row_height = ui.text_style_height(&text_style);

		let address_range = memory.address_range().clone();
		let total_rows = (address_range.len() + self.column_count - 1) / self.column_count;

		scroll.show_rows(ui, row_height, total_rows, |ui, row_range| {
			egui::Grid::new("memory_dump").striped(true).show(ui, |ui| {
				ui.style_mut().wrap = Some(false);

				for row in row_range.clone() {
					let start_address =
						address_range.start + ((row as u16) * self.column_count as u16);

					ui.label(format!("0x{:04X}:\t\t", start_address));

					self.draw_memory_values(ui, memory, address_range.clone(), start_address);
					self.draw_ascii_values(ui, memory, address_range.clone(), start_address);

					ui.end_row();
				}
			});
		});
	}

	fn draw_memory_values(
		&self,
		ui: &mut egui::Ui,
		memory: &mut impl Memory,
		address_range: Range<u16>,
		start_address: u16,
	) {
		for i in 0..self.column_count {
			let address = start_address + (i as u16);
			if !address_range.contains(&address) {
				ui.label("00 00");
				break;
			}
			let value = memory.read(start_address + (i as u16));
			let low_byte = (value & 0xFF) as u8;
			let high_byte = ((value >> 8) & 0xFF) as u8;

			ui.label(format!("{:02X} {:02X}", high_byte, low_byte));
		}
	}

	fn draw_ascii_values(
		&self,
		ui: &mut egui::Ui,
		memory: &mut impl Memory,
		address_range: Range<u16>,
		start_address: u16,
	) {
		ui.horizontal(|ui| {
			ui.add(egui::Separator::default().vertical().spacing(3.0));
			ui.style_mut().spacing.item_spacing.x = 0.0;

			ui.horizontal(|ui| {
				for i in 0..self.column_count {
					let address = start_address + (i as u16);
					if !address_range.contains(&address) {
						ui.label(egui::RichText::new('.').text_style(egui::TextStyle::Monospace));
						ui.label(egui::RichText::new('.').text_style(egui::TextStyle::Monospace));
						break;
					}
					let value = memory.read(start_address + (i as u16));
					let low_byte = (value & 0xFF) as u8;
					let high_byte = ((value >> 8) & 0xFF) as u8;

					let chars = match (
						!(32..128).contains(&low_byte),
						!(32..128).contains(&high_byte),
					) {
						(true, true) => ('.', '.'),
						(false, false) => (low_byte as char, high_byte as char),
						(true, false) => ('.', high_byte as char),
						(false, true) => (low_byte as char, '.'),
					};

					ui.label(egui::RichText::new(chars.1).text_style(egui::TextStyle::Monospace));
					ui.label(egui::RichText::new(chars.0).text_style(egui::TextStyle::Monospace));
				}
			});
		});
	}
}

pub struct MemoryView {
	selected_tab: Tab,
	memory_tab: MemoryTab,
}

impl Default for MemoryView {
	fn default() -> Self {
		Self {
			selected_tab: Tab::ProgramFlash,
			memory_tab: MemoryTab::default(),
		}
	}
}

impl MemoryView {
	pub fn ui(&mut self, ui: &mut egui::Ui, cpu: &mut Cpu) {
		ui.add_space(PADDING_SIZE);
		ui.horizontal(|ui| {
			ui.selectable_value(&mut self.selected_tab, Tab::ProgramFlash, "Program Flash");
			ui.selectable_value(&mut self.selected_tab, Tab::DataMemory, "Data Memory");
			ui.selectable_value(&mut self.selected_tab, Tab::Eeprom, "EEPROM");
		});

		ui.separator();

		match self.selected_tab {
			Tab::ProgramFlash => {
				self.memory_tab.ui(ui, &mut cpu.system.program_memory);
			}
			Tab::DataMemory => {
				self.memory_tab.ui(ui, &mut cpu.sram);
			}
			Tab::Eeprom => {
				self.memory_tab.ui(ui, &mut cpu.system.eeprom_memory);
			}
		}
	}
}
