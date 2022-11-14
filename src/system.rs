use crate::memory::{EepromMemory, ProgramMemory};

#[derive(Default)]
pub struct System {
	pub program_memory: ProgramMemory,
	pub eeprom_memory: EepromMemory,
}
