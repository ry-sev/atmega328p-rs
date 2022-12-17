use crate::memory::{EepromMemory, Memory, ProgramMemory, PROGRAM_START};

#[derive(Default)]
pub struct System {
	pub program_memory: ProgramMemory,
	pub eeprom_memory: EepromMemory,
}

impl System {
	pub fn flash_from_vec(&mut self, program: Vec<u16>) {
		for (index, word) in program.into_iter().enumerate() {
			self.program_memory
				.write(PROGRAM_START + (index as u16), word);
		}
	}
}
