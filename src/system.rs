use crate::{
	disassembler::Disassembler,
	memory::{EepromMemory, Memory, ProgramMemory, PROGRAM_END, PROGRAM_START},
};

#[derive(Default)]
pub struct System {
	pub program_memory: ProgramMemory,
	pub eeprom_memory: EepromMemory,
	pub disassembler: Disassembler,
}

impl System {
	pub fn flash_from_vec(&mut self, program: Vec<u16>) {
		for (index, word) in program.into_iter().enumerate() {
			self.program_memory
				.write(PROGRAM_START + (index as u16), word);
		}
		self.disassembler.disassemble(
			&mut self.program_memory.app_flash,
			PROGRAM_START,
			PROGRAM_END,
		);
	}
}
