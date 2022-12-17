use crate::{
	cpu::Cpu,
	memory::{Memory, REGISTER_NAMES},
};
use egui_extras::{Column, TableBuilder};

const PADDING_SIZE: f32 = 4.0;

fn status_color(set: bool) -> egui::Color32 {
	if set {
		egui::Color32::GREEN
	} else {
		egui::Color32::RED
	}
}

#[derive(PartialEq, Eq)]
enum Tab {
	Registers,
	IORegisters,
	ExtRegisters,
}

#[derive(Default)]
struct RegisterTab {}

impl RegisterTab {
	fn ui(&mut self, ui: &mut egui::Ui, registers: &Vec<u8>, start_index: u8, subract: u8) {
		let table = TableBuilder::new(ui)
			.striped(true)
			.cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
			.column(Column::exact(50.0))
			.column(Column::exact(60.0))
			.column(Column::remainder())
			.resizable(false);

		table
			.header(20.0, |mut header| {
				header.col(|ui| {
					ui.label("Address");
				});
				header.col(|ui| {
					ui.label("Name");
				});
				header.col(|ui| {
					ui.label("Value");
				});
			})
			.body(|mut body| {
				for address in 0x00..(registers.len() as u8) - subract {
					let name = REGISTER_NAMES
						.get(&(address + start_index))
						.expect("Register address does not exist");

					if name.eq("Reserved") {
						continue;
					}

					let value = registers[address as usize];

					body.row(18.0, |mut row| {
						row.col(|ui| {
							ui.label(format!("0x{:02X}", address + start_index));
						});
						row.col(|ui| {
							ui.label(name);
						});
						row.col(|ui| {
							ui.label(format!("0x{:02X}  [{}]", value, value));
						});
					});
				}
			});
	}
}

pub struct CpuState {
	selected_tab: Tab,
	register_tab: RegisterTab,
}

impl Default for CpuState {
	fn default() -> Self {
		Self {
			selected_tab: Tab::Registers,
			register_tab: RegisterTab::default(),
		}
	}
}

impl CpuState {
	pub fn ui(&mut self, ui: &mut egui::Ui, cpu: &mut Cpu) {
		ui.add_space(PADDING_SIZE);

		ui.label("CPU State");

		ui.separator();

		egui::Grid::new("cpu_state")
			.num_columns(2)
			.spacing([50.0, 4.0])
			.striped(true)
			.show(ui, |ui| {
				ui.label("Status: ");
				ui.horizontal(|ui| {
					ui.colored_label(status_color(cpu.status.I), "I");
					ui.colored_label(status_color(cpu.status.T), "T");
					ui.colored_label(status_color(cpu.status.H), "H");
					ui.colored_label(status_color(cpu.status.S), "S");
					ui.colored_label(status_color(cpu.status.V), "V");
					ui.colored_label(status_color(cpu.status.N), "N");
					ui.colored_label(status_color(cpu.status.Z), "Z");
					ui.colored_label(status_color(cpu.status.C), "C");
				});

				ui.end_row();

				ui.label("Program Counter:");
				ui.label(format!("0x{:04X}", cpu.pc));

				ui.end_row();

				ui.label("Instruction:");
				ui.label(format!("0x{:04X}", cpu.system.program_memory.read(cpu.pc)));

				ui.end_row();

				ui.label("Stack Pointer:");
				ui.label(format!("0x{:04X}", cpu.sp));

				ui.end_row();

				ui.label("X Register:");
				let x_reg =
					((cpu.sram.registers[27] as u16) << 8) | (cpu.sram.registers[26] as u16);
				ui.label(format!("0x{:04X}", x_reg));

				ui.end_row();

				ui.label("Y Register:");
				let y_reg =
					((cpu.sram.registers[29] as u16) << 8) | (cpu.sram.registers[28] as u16);
				ui.label(format!("0x{:04X}", y_reg));

				ui.end_row();

				ui.label("Z Register:");
				let y_reg =
					((cpu.sram.registers[31] as u16) << 8) | (cpu.sram.registers[30] as u16);
				ui.label(format!("0x{:04X}", y_reg));

				ui.end_row();

				ui.label("Cycle Counter:");
				ui.label(format!("{}", cpu.cycles));

				ui.end_row();

				ui.label("Frequency:");

				ui.end_row();

				ui.label("Stop Watch:");
			});

		ui.separator();

		ui.horizontal(|ui| {
			ui.selectable_value(&mut self.selected_tab, Tab::Registers, "Registers");
			ui.selectable_value(&mut self.selected_tab, Tab::IORegisters, "I/O Registers");
			ui.selectable_value(
				&mut self.selected_tab,
				Tab::ExtRegisters,
				"Ext. I/O Registers",
			);
		});

		ui.separator();

		match self.selected_tab {
			Tab::Registers => {
				self.register_tab.ui(ui, &cpu.sram.registers, 0x00, 0);
			}
			Tab::IORegisters => {
				self.register_tab.ui(ui, &cpu.sram.io_registers, 0x20, 3);
			}
			Tab::ExtRegisters => {
				self.register_tab
					.ui(ui, &cpu.sram.ext_io_registers, 0x60, 0);
			}
		}
	}
}
