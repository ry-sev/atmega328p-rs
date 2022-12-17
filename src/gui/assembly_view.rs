use crate::disassembler::InstructionString;
use egui_extras::{Column, TableBuilder};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct AssemblyView {}

impl AssemblyView {
	pub fn ui(&mut self, ui: &mut egui::Ui, assembly: &BTreeMap<u16, InstructionString>) {
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
							ui.label(&instruction.address);
						});
						row.col(|ui| {
							ui.label(&instruction.opcode);
						});
						row.col(|ui| {
							ui.label(&instruction.instruction);
						});
					});
				}
			});
	}
}
