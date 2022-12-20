use regex::Regex;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;

use crate::{
	disassembler::Disassembler,
	memory::{EepromMemory, Memory, ProgramMemory, PROGRAM_START},
};

#[derive(Default)]
pub struct System {
	pub program_memory: ProgramMemory,
	pub eeprom_memory: EepromMemory,
	pub disassembler: Disassembler,
	pub last_instuction_address: u16,
}

impl System {
	#[cfg(test)]
	pub fn flash_from_vec(&mut self, program: Vec<u16>) {
		let program_length = program.len() as u16;
		for (index, word) in program.into_iter().enumerate() {
			self.program_memory
				.write(PROGRAM_START + (index as u16), word);
		}
		self.disassembler.disassemble(
			&mut self.program_memory.app_flash,
			PROGRAM_START,
			program_length,
		);
	}

	pub fn flash_from_hex_file(&mut self, program_file: &PathBuf) {
		let re = match Regex::new(
			r":(?P<data_size>[A-z0-9]{2})(?P<start_address>[A-z0-9]{4})(?P<record_type>[A-z0-9]{2})(?P<data>[A-z0-9]+)(?P<checksum>[A-z0-9]{2})$",
		) {
			Err(_) => {
				println!("Invalid regex string for hex file parsing");
				return;
			}
			Ok(rgx) => rgx,
		};

		let file = match File::open(program_file) {
			Err(_) => {
				println!("Unable to open .hex file: {}", program_file.display());
				return;
			}
			Ok(f) => f,
		};

		let reader = BufReader::new(file);
		let lines: Vec<_> = reader.lines().map(|line| line.unwrap()).collect();

		self.program_memory.app_flash.clear();
		let mut program_length: u16 = 0;

		for line in lines.iter() {
			match re.captures(line) {
				None => continue,
				Some(capture) => {
					let chars: Vec<char> = capture["data"].chars().to_owned().collect();

					for x in 0..(chars.len() / 4) {
						let index = x * 4;
						let a = chars[index + 2].to_digit(16).unwrap() as u16;
						let b = chars[index + 3].to_digit(16).unwrap() as u16;
						let c = chars[index].to_digit(16).unwrap() as u16;
						let d = chars[index + 1].to_digit(16).unwrap() as u16;

						let word = ((a << 12) | (b << 8)) | ((c << 4) | d);
						program_length += 1;
						self.program_memory
							.write(PROGRAM_START + program_length, word);
					}
				}
			}
		}

		self.disassembler.disassemble(
			&mut self.program_memory.app_flash,
			PROGRAM_START,
			program_length,
		);
	}
}
