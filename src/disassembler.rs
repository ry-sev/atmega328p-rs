use crate::memory::{ApplicationFlash, Memory};
use crate::utils;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub struct InstructionString {
	pub address: u16,
	pub opcode: u16,
	pub instruction: String,
}

#[derive(Debug)]
pub struct Disassembler {
	pub assembly: Option<BTreeMap<u16, InstructionString>>,
	opcode: u16,
}

impl Default for Disassembler {
	fn default() -> Self {
		Self {
			assembly: None,
			opcode: 0x0000,
		}
	}
}

impl Disassembler {
	fn create_string_with_two_registers(
		&self,
		match_start: u8,
		instruction: String,
		inst_string: &mut String,
	) {
		let mut destination = ((self.opcode & 0xF0) >> 4) as u8;
		let mut source = (self.opcode & 0xF) as u8;
		let num_2 = match_start + 1;
		let num_3 = match_start + 2;
		match utils::high_byte(self.opcode) {
			match_start => destination += 16,
			num_2 => source += 16,
			num_3 => {
				destination += 16;
				source += 16;
			}
			_ => {}
		}
		inst_string.push_str(format!("{} r{}, r{}", instruction, destination, source).as_str());
	}

	fn create_string_with_two_registers_2(
		&self,
		and_value_1: u16,
		and_value_2: u16,
		instruction: String,
		inst_string: &mut String,
	) {
		let destination = (((self.opcode & and_value_1) >> 4) as u8) + 16;
		let source = ((self.opcode & and_value_2) as u8) + 16;
		inst_string.push_str(format!("{} r{}, r{}", instruction, destination, source).as_str());
	}

	fn create_string_with_register_and_constant(
		&self,
		instruction: String,
		inst_string: &mut String,
	) {
		let destination = (((self.opcode & 0xF0) >> 4) as u8) + 16;
		let value = ((((self.opcode >> 8) & 0xF) << 4) | (self.opcode & 0xF)) as u8;
		inst_string.push_str(
			format!(
				"{} r{}, 0x{:02X} [{}]",
				instruction, destination, value, value
			)
			.as_str(),
		);
	}

	fn create_string_with_registers_and_word(&self, instruction: String, inst_string: &mut String) {
		let mut value = self.opcode & 0xF;
		value |= (self.opcode & (1 << 6)) >> 2;
		value |= (self.opcode & (1 << 7)) >> 2;
		let destination = (((self.opcode >> 4 & 0xF) & 0x3) * 2 + 24) as u8;
		inst_string.push_str(
			format!(
				"{} r{}:r{}, 0x{:02X} [{}]",
				instruction,
				destination + 1,
				destination,
				value,
				value
			)
			.as_str(),
		);
	}

	pub fn disassemble(
		&mut self,
		program: &mut ApplicationFlash,
		start_address: u16,
		end_address: u16,
	) {
		let mut assembly: BTreeMap<u16, InstructionString> = BTreeMap::new();
		let mut current_address = start_address;

		while current_address < end_address {
			let id = current_address;
			let address = current_address;
			self.opcode = program.read(current_address);
			let opcode = self.opcode;

			let low_byte = (self.opcode & 0xF) as u8;
			let high_byte = ((self.opcode >> 4) & 0xF) as u8;

			let mut instruction = String::new();

			match self.opcode {
				0x0000..=0x00FF => match (self.opcode & 0xFF) as u8 {
					0x00 => instruction.push_str("nop"),
					_ => instruction.push_str("[R]"),
				},
				0x0100..=0x01FF => {
					//movw
				}
				0x0200..=0x02FF => {
					self.create_string_with_two_registers_2(
						0xF0,
						0xF,
						"muls".to_string(),
						&mut instruction,
					);
				}
				0x0300..=0x03FF => match low_byte {
					0x0..=0x7 => match high_byte {
						0x0..=0x7 => {
							self.create_string_with_two_registers_2(
								0x70,
								0x7,
								"mulsu".to_string(),
								&mut instruction,
							);
						}
						0x8..=0xF => {
							self.create_string_with_two_registers_2(
								0x70,
								0x7,
								"fmuls".to_string(),
								&mut instruction,
							);
						}
						_ => unreachable!(),
					},
					0x8..=0xF => match high_byte {
						0x0..=0x7 => {
							self.create_string_with_two_registers_2(
								0x70,
								0x7,
								"fmul".to_string(),
								&mut instruction,
							);
						}
						0x8..=0xF => {
							self.create_string_with_two_registers_2(
								0x70,
								0x7,
								"fmulsu".to_string(),
								&mut instruction,
							);
						}
						_ => unreachable!(),
					},
					_ => unreachable!(),
				},
				0x0400..=0x07FF => {
					//cpc
				}
				0x0800..=0x0BFF => {
					self.create_string_with_two_registers(
						0x09,
						"sbc".to_string(),
						&mut instruction,
					);
				}
				0x0C00..=0x0FFF => {
					self.create_string_with_two_registers(
						0x0D,
						"add".to_string(),
						&mut instruction,
					);
				}
				0x1000..=0x13FF => {
					//cpse
				}
				0x1400..=0x17FF => {
					//cp
				}
				0x1800..=0x1BFF => {
					self.create_string_with_two_registers(
						0x19,
						"sub".to_string(),
						&mut instruction,
					);
				}
				0x1C00..=0x1FFF => {
					self.create_string_with_two_registers(
						0x1D,
						"adc".to_string(),
						&mut instruction,
					);
				}
				0x2000..=0x23FF => {
					self.create_string_with_two_registers(
						0x21,
						"and".to_string(),
						&mut instruction,
					);
				}
				0x2400..=0x27FF => {
					self.create_string_with_two_registers(
						0x25,
						"eor".to_string(),
						&mut instruction,
					);
				}
				0x2800..=0x2BFF => {
					self.create_string_with_two_registers(0x29, "or".to_string(), &mut instruction);
				}
				0x2C00..=0x2FFF => {
					//mov
				}
				0x3000..=0x3FFF => {
					//cpi
				}
				0x4000..=0x4FFF => {
					self.create_string_with_register_and_constant(
						"sbci".to_string(),
						&mut instruction,
					);
				}
				0x5000..=0x5FFF => {
					self.create_string_with_register_and_constant(
						"subi".to_string(),
						&mut instruction,
					);
				}
				0x6000..=0x6FFF => {
					self.create_string_with_register_and_constant(
						"ori".to_string(),
						&mut instruction,
					);
				}
				0x7000..=0x7FFF => {
					self.create_string_with_register_and_constant(
						"andi".to_string(),
						&mut instruction,
					);
				}
				0x8000..=0x81FF => {
					//ldd
				}
				0x8200..=0x83FF => {
					//std
				}
				0x8400..=0x85FF => {
					//ldd
				}
				0x8600..=0x87FF => {
					//std
				}
				0x8800..=0x89FF => {
					//ldd
				}
				0x8A00..=0x8BFF => {
					//std
				}
				0x8C00..=0x8DFF => {
					//ldd
				}
				0x8E00..=0x8FFF => {
					//std
				}
				0x9000..=0x91FF => match low_byte {
					0x0 => {
						//lds
					}
					0x1..=0x2 => {
						//ld_z
					}
					0x3 => instruction.push_str("[R]"),
					0x4..=0x5 => {
						//lpm
					}
					0x6..=0x8 => instruction.push_str("[R]"),
					0x9..=0xA => {
						//ld_y
					}
					0xB => instruction.push_str("[R]"),
					0xC..=0xE => {
						//ld_x
					}
					0xF => {
						//pop
					}
					_ => unreachable!(),
				},
				0x9200..=0x93FF => match low_byte {
					0x0 => {
						//sts
					}
					0x1..=0x2 => {
						//st_z
					}
					0x3..=0x8 => instruction.push_str("[R]"),
					0x9..=0xA => {
						//st_y
					}
					0xB => instruction.push_str("[R]"),
					0xC..=0xE => {
						//st_x
					}
					0xF => {
						//push
					}
					_ => unreachable!(),
				},
				0x9400..=0x94FF => match low_byte {
					0x0 => {
						let source = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str(format!("com r{}", source).as_str());
					}
					0x1 => {
						let source = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str(format!("neg r{}", source).as_str());
					}
					0x2 => {
						//swap
					}
					0x3 => {
						//inc
					}
					0x4 => instruction.push_str("[R]"),
					0x5 => {
						//asr
					}
					0x6 => {
						//lsr
					}
					0x7 => {
						//ror
					}
					0x8 => match high_byte {
						0x0 => instruction.push_str("sec"),
						0x1 => instruction.push_str("sez"),
						0x2 => instruction.push_str("sen"),
						0x3 => instruction.push_str("sev"),
						0x4 => instruction.push_str("ses"),
						0x5 => instruction.push_str("seh"),
						0x6 => instruction.push_str("set"),
						0x7 => instruction.push_str("sei"),
						0x8 => instruction.push_str("clc"),
						0x9 => instruction.push_str("clz"),
						0xA => instruction.push_str("cln"),
						0xB => instruction.push_str("clv"),
						0xC => instruction.push_str("cls"),
						0xD => instruction.push_str("clh"),
						0xE => instruction.push_str("clt"),
						0xF => instruction.push_str("cli"),
						_ => unreachable!(),
					},
					0x9 => match high_byte {
						0x0 => instruction.push_str("ijmp"),
						_ => instruction.push_str("[R]"),
					},
					0xA => {
						let source = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str(format!("dec r{}", source).as_str());
					}
					0xB => {
						let value = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str(format!("des 0x{:02X} [{}]", value, value).as_str());
					}
					0xC..=0xD => {
						//jmp
					}
					0xE..=0xF => {
						//call
					}
					_ => unreachable!(),
				},
				0x9500..=0x95FF => match low_byte {
					0x00 => {
						let source = (((self.opcode & 0xF0) >> 4) as u8) + 16;
						instruction.push_str(format!("com r{}", source).as_str());
					}
					0x01 => {
						let source = (((self.opcode & 0xF0) >> 4) as u8) + 16;
						instruction.push_str(format!("neg r{}", source).as_str());
					}
					0x02 => {
						//swap
					}
					0x03 => {
						//inc
					}
					0x04 => instruction.push_str("[R]"),
					0x05 => {
						//asr
					}
					0x06 => {
						//lsr
					}
					0x07 => {
						//ror
					}
					0x08 => match high_byte {
						0x0 => instruction.push_str("ret"),
						0x1 => instruction.push_str("reti"),
						0x8 => instruction.push_str("sleep"),
						0x9 => instruction.push_str("break"),
						0xA => instruction.push_str("wdr"),
						0xC => {
							//lpm
						}
						0xE..=0xF => {
							//spm
						}
						_ => instruction.push_str("[R]"),
					},
					0x09 => match high_byte {
						0x0 => instruction.push_str("icall"),
						_ => instruction.push_str("[R]"),
					},
					0x0A => {
						let source = (((self.opcode & 0xF0) >> 4) as u8) + 16;
						instruction.push_str(format!("dec r{}", source).as_str());
					}
					0x0B => instruction.push_str("[R]"),
					0xC..=0xD => {
						//jmp
					}
					0x0E..=0x0F => {
						//call
					}
					_ => unreachable!(),
				},
				0x9600..=0x96FF => {
					self.create_string_with_registers_and_word(
						"adiw".to_string(),
						&mut instruction,
					);
				}
				0x9700..=0x97FF => {
					self.create_string_with_registers_and_word(
						"sbiw".to_string(),
						&mut instruction,
					);
				}
				0x9800..=0x98FF => {
					//cbi
				}
				0x9900..=0x99FF => {
					//sbic
				}
				0x9A00..=0x9AFF => {
					//sbi
				}
				0x9B00..=0x9BFF => {
					//sbis
				}
				0x9C00..=0x9FFF => {
					self.create_string_with_two_registers(
						0x9D,
						"mul".to_string(),
						&mut instruction,
					);
				}
				0xA000..=0xA1FF => {
					//ldd
				}
				0xA200..=0xA3FF => {
					//std
				}
				0xA400..=0xA5FF => {
					//ldd
				}
				0xA600..=0xA7FF => {
					//std
				}
				0xA800..=0xA9FF => {
					//ldd
				}
				0xAA00..=0xABFF => {
					//std
				}
				0xAC00..=0xADFF => {
					//ldd
				}
				0xAE00..=0xAFFF => {
					//std
				}
				0xB000..=0xB7FF => {
					//in_
				}
				0xB800..=0xBFFF => {
					//out
				}
				0xC000..=0xCFFF => {
					//rjmp
				}
				0xD000..=0xDFFF => {
					//rcall
				}
				0xE000..=0xEFFF => {
					//ldi
				}
				0xF000..=0xF3FF => match low_byte {
					0x0 => {
						//brcs
					}
					0x1 => {
						//breq
					}
					0x2 => {
						//brmi
					}
					0x3 => {
						//brvs
					}
					0x4 => {
						//brlt
					}
					0x5 => {
						//brhs
					}
					0x6 => {
						//brts
					}
					0x7 => {
						//brie
					}
					0x8 => {
						//brcs
					}
					0x9 => {
						//breq
					}
					0xA => {
						//brmi
					}
					0xB => {
						//brvs
					}
					0xC => {
						//brlt
					}
					0xD => {
						//brhs
					}
					0xE => {
						//brts
					}
					0xF => {
						//brie
					}
					_ => unreachable!(),
				},
				0xF400..=0xF7FF => match low_byte {
					0x0 => {
						//brcc
					}
					0x1 => {
						//brne
					}
					0x2 => {
						//brpl
					}
					0x3 => {
						//brvc
					}
					0x4 => {
						//brge
					}
					0x5 => {
						//brhc
					}
					0x6 => {
						//brtc
					}
					0x7 => {
						//brid
					}
					0x8 => {
						//brcc
					}
					0x9 => {
						//brne
					}
					0xA => {
						//brpl
					}
					0xB => {
						//brvc
					}
					0xC => {
						//brge
					}
					0xD => {
						//brhc
					}
					0xE => {
						//brtc
					}
					0xF => {
						//brid
					}
					_ => unreachable!(),
				},
				0xF800..=0xF9FF => {
					//bld
				}
				0xFA00..=0xFBFF => {
					//bst
				}
				0xFC00..=0xFDFF => {
					//sbrc
				}
				0xFE00..=0xFFFF => {
					//sbrs
				}
			}
			assembly.insert(
				id,
				InstructionString {
					address,
					opcode,
					instruction,
				},
			);
			current_address += 1;
		}
		self.assembly = Some(assembly);
	}
}
