use crate::disassembler::Instruction;
use egui_extras::{Column, TableBuilder};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct AssemblyView {}

impl AssemblyView {
	pub fn ui(
		&mut self,
		ui: &mut egui::Ui,
		assembly: &BTreeMap<u16, Instruction>,
		program_counter: &u16,
	) {
		let table = TableBuilder::new(ui)
			.striped(true)
			.cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
			.column(Column::exact(60.0))
			.column(Column::exact(60.0))
			.column(Column::remainder())
			.resizable(false);

		table
			.header(20.0, |mut header| {
				header.col(|ui| {
					ui.label("Address");
				});
				header.col(|ui| {
					ui.label("Opcode");
				});
				header.col(|ui| {
					ui.label("Instruction");
				});
			})
			.body(|mut body| {
				for (_, instruction) in assembly {
					body.row(18.0, |mut row| {
						row.col(|ui| {
							if &instruction.address == program_counter {
								ui.code(format!("0x{:04X}", &instruction.address));
							} else {
								ui.label(format!("0x{:04X}", &instruction.address));
							}
						});
						row.col(|ui| {
							ui.label(format!("0x{:04X}", &instruction.opcode));
						});
						row.col(|ui| {
							ui.colored_label(egui::Color32::LIGHT_RED, &instruction.instruction);
							ui.label(&instruction.operands);
						});
					});
				}
			});
	}
}
