use crate::memory::{ApplicationFlash, Memory};
use crate::utils;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Instruction {
	pub address: u16,
	pub opcode: u16,
	pub instruction: String,
	pub operands: String,
}

#[derive(Debug)]
pub struct Disassembler {
	pub assembly: Option<BTreeMap<u16, Instruction>>,
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
	fn create_string_with_two_registers(&self, match_start: u8, operands_str: &mut String) {
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
		operands_str.push_str(format!("r{}, r{}", destination, source).as_str());
	}

	fn create_string_with_two_registers_2(
		&self,
		and_value_1: u16,
		and_value_2: u16,

		operands_str: &mut String,
	) {
		let destination = (((self.opcode & and_value_1) >> 4) as u8) + 16;
		let source = ((self.opcode & and_value_2) as u8) + 16;
		operands_str.push_str(format!("r{}, r{}", destination, source).as_str());
	}

	fn create_string_with_register_and_constant(&self, operands_str: &mut String) {
		let destination = (((self.opcode & 0xF0) >> 4) as u8) + 16;
		let value = ((((self.opcode >> 8) & 0xF) << 4) | (self.opcode & 0xF)) as u8;
		operands_str.push_str(format!("r{}, 0x{:02X} [{}]", destination, value, value).as_str());
	}

	fn create_string_with_registers_and_word(&self, operands_str: &mut String) {
		let mut value = self.opcode & 0xF;
		value |= (self.opcode & (1 << 6)) >> 2;
		value |= (self.opcode & (1 << 7)) >> 2;
		let destination = (((self.opcode >> 4 & 0xF) & 0x3) * 2 + 24) as u8;
		operands_str.push_str(
			format!(
				"r{}:r{}, 0x{:02X} [{}]",
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
		let mut assembly: BTreeMap<u16, Instruction> = BTreeMap::new();
		let mut current_address = start_address;

		while current_address < end_address {
			let id = current_address;
			let address = current_address;
			self.opcode = program.read(current_address);
			let opcode = self.opcode;

			let low_byte = (self.opcode & 0xF) as u8;
			let high_byte = ((self.opcode >> 4) & 0xF) as u8;

			let mut instruction = String::new();
			let mut operands = String::new();

			match self.opcode {
				0x0000..=0x00FF => match (self.opcode & 0xFF) as u8 {
					0x00 => instruction.push_str("nop"),
					_ => instruction.push_str("[R]"),
				},
				0x0100..=0x01FF => {
					instruction.push_str("movw");
				}
				0x0200..=0x02FF => {
					instruction.push_str("muls");
					self.create_string_with_two_registers_2(0xF0, 0xF, &mut operands);
				}
				0x0300..=0x03FF => match low_byte {
					0x0..=0x7 => match high_byte {
						0x0..=0x7 => {
							instruction.push_str("mulsu");
							self.create_string_with_two_registers_2(0x70, 0x7, &mut operands);
						}
						0x8..=0xF => {
							instruction.push_str("fmuls");
							self.create_string_with_two_registers_2(0x70, 0x7, &mut operands);
						}
						_ => unreachable!(),
					},
					0x8..=0xF => match high_byte {
						0x0..=0x7 => {
							instruction.push_str("fmul");
							self.create_string_with_two_registers_2(0x70, 0x7, &mut operands);
						}
						0x8..=0xF => {
							instruction.push_str("fmulsu");
							self.create_string_with_two_registers_2(0x70, 0x7, &mut operands);
						}
						_ => unreachable!(),
					},
					_ => unreachable!(),
				},
				0x0400..=0x07FF => {
					instruction.push_str("cpc");
					self.create_string_with_two_registers(0x05, &mut operands);
				}
				0x0800..=0x0BFF => {
					instruction.push_str("sbc");
					self.create_string_with_two_registers(0x09, &mut operands);
				}
				0x0C00..=0x0FFF => {
					instruction.push_str("add");
					self.create_string_with_two_registers(0x0D, &mut operands);
				}
				0x1000..=0x13FF => {
					instruction.push_str("cpse");
					self.create_string_with_two_registers(0x11, &mut operands);
				}
				0x1400..=0x17FF => {
					instruction.push_str("cp");
					self.create_string_with_two_registers(0x15, &mut operands);
				}
				0x1800..=0x1BFF => {
					instruction.push_str("sub");
					self.create_string_with_two_registers(0x19, &mut operands);
				}
				0x1C00..=0x1FFF => {
					instruction.push_str("adc");
					self.create_string_with_two_registers(0x1D, &mut operands);
				}
				0x2000..=0x23FF => {
					instruction.push_str("and");
					self.create_string_with_two_registers(0x21, &mut operands);
				}
				0x2400..=0x27FF => {
					instruction.push_str("eor");
					self.create_string_with_two_registers(0x25, &mut operands);
				}
				0x2800..=0x2BFF => {
					instruction.push_str("or");
					self.create_string_with_two_registers(0x29, &mut operands);
				}
				0x2C00..=0x2FFF => {
					instruction.push_str("mov");
					self.create_string_with_two_registers(0x2D, &mut operands);
				}
				0x3000..=0x3FFF => {
					instruction.push_str("cpi");
					self.create_string_with_register_and_constant(&mut operands);
				}
				0x4000..=0x4FFF => {
					instruction.push_str("sbci");
					self.create_string_with_register_and_constant(&mut operands);
				}
				0x5000..=0x5FFF => {
					instruction.push_str("subi");
					self.create_string_with_register_and_constant(&mut operands);
				}
				0x6000..=0x6FFF => {
					instruction.push_str("ori");
					self.create_string_with_register_and_constant(&mut operands);
				}
				0x7000..=0x7FFF => {
					instruction.push_str("andi");
					self.create_string_with_register_and_constant(&mut operands);
				}
				0x8000..=0x81FF => {
					instruction.push_str("ld_z");
					instruction.push_str("ld_y");
					instruction.push_str("ldd");
				}
				0x8200..=0x83FF => {
					instruction.push_str("std");
				}
				0x8400..=0x85FF => {
					instruction.push_str("ldd");
				}
				0x8600..=0x87FF => {
					instruction.push_str("std");
				}
				0x8800..=0x89FF => {
					instruction.push_str("ldd");
				}
				0x8A00..=0x8BFF => {
					instruction.push_str("std");
				}
				0x8C00..=0x8DFF => {
					instruction.push_str("ldd");
				}
				0x8E00..=0x8FFF => {
					instruction.push_str("std");
				}
				0x9000..=0x91FF => match low_byte {
					0x0 => {
						instruction.push_str("lds");
					}
					0x1..=0x2 => {
						instruction.push_str("ld_z");
					}
					0x3 => instruction.push_str("[R]"),
					0x4..=0x5 => {
						instruction.push_str("lpm");
					}
					0x6..=0x8 => instruction.push_str("[R]"),
					0x9..=0xA => {
						instruction.push_str("ld_y");
					}
					0xB => instruction.push_str("[R]"),
					0xC..=0xE => {
						instruction.push_str("ld_x");
					}
					0xF => {
						instruction.push_str("pop");
					}
					_ => unreachable!(),
				},
				0x9200..=0x93FF => match low_byte {
					0x0 => {
						instruction.push_str("sts");
					}
					0x1..=0x2 => {
						instruction.push_str("st_z");
					}
					0x3..=0x8 => instruction.push_str("[R]"),
					0x9..=0xA => {
						instruction.push_str("st_y");
					}
					0xB => instruction.push_str("[R]"),
					0xC..=0xE => {
						instruction.push_str("st_x");
					}
					0xF => {
						instruction.push_str("push");
					}
					_ => unreachable!(),
				},
				0x9400..=0x94FF => match low_byte {
					0x0 => {
						let source = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str("com");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x1 => {
						let source = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str("neg");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x2 => {
						let source = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str("swap");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x3 => {
						let source = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str("inc");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x4 => instruction.push_str("[R]"),
					0x5 => {
						let source = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str("asr");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x6 => {
						let source = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str("lsr");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x7 => {
						let source = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str("ror");
						operands.push_str(format!("r{}", source).as_str());
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
						instruction.push_str("dec");
						operands.push_str(format!("r{}", source).as_str());
					}
					0xB => {
						let value = ((self.opcode & 0xF0) >> 4) as u8;
						instruction.push_str("des");
						operands.push_str(format!("0x{:02X} [{}]", value, value).as_str());
					}
					0xC..=0xD => {
						instruction.push_str("jmp");
					}
					0xE..=0xF => {
						instruction.push_str("call");
					}
					_ => unreachable!(),
				},
				0x9500..=0x95FF => match low_byte {
					0x00 => {
						let source = (((self.opcode & 0xF0) >> 4) as u8) + 16;
						instruction.push_str("com");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x01 => {
						let source = (((self.opcode & 0xF0) >> 4) as u8) + 16;
						instruction.push_str("neg");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x02 => {
						let source = (((self.opcode & 0xF0) >> 4) as u8) + 16;
						instruction.push_str("swap");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x03 => {
						let source = (((self.opcode & 0xF0) >> 4) as u8) + 16;
						instruction.push_str("inc");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x04 => instruction.push_str("[R]"),
					0x05 => {
						let source = (((self.opcode & 0xF0) >> 4) as u8) + 16;
						instruction.push_str("asr");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x06 => {
						let source = (((self.opcode & 0xF0) >> 4) as u8) + 16;
						instruction.push_str("lsr");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x07 => {
						let source = (((self.opcode & 0xF0) >> 4) as u8) + 16;
						instruction.push_str("ror");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x08 => match high_byte {
						0x0 => instruction.push_str("ret"),
						0x1 => instruction.push_str("reti"),
						0x8 => instruction.push_str("sleep"),
						0x9 => instruction.push_str("break"),
						0xA => instruction.push_str("wdr"),
						0xC => {
							instruction.push_str("lpm");
						}
						0xE..=0xF => {
							instruction.push_str("spm");
						}
						_ => instruction.push_str("[R]"),
					},
					0x09 => match high_byte {
						0x0 => instruction.push_str("icall"),
						_ => instruction.push_str("[R]"),
					},
					0x0A => {
						let source = (((self.opcode & 0xF0) >> 4) as u8) + 16;
						instruction.push_str("dec");
						operands.push_str(format!("r{}", source).as_str());
					}
					0x0B => instruction.push_str("[R]"),
					0xC..=0xD => {
						instruction.push_str("jmp");
					}
					0x0E..=0x0F => {
						instruction.push_str("call");
					}
					_ => unreachable!(),
				},
				0x9600..=0x96FF => {
					instruction.push_str("adiw");
					self.create_string_with_registers_and_word(&mut operands);
				}
				0x9700..=0x97FF => {
					instruction.push_str("sbiw");
					self.create_string_with_registers_and_word(&mut operands);
				}
				0x9800..=0x98FF => {
					instruction.push_str("cbi");
				}
				0x9900..=0x99FF => {
					instruction.push_str("sbic");
				}
				0x9A00..=0x9AFF => {
					instruction.push_str("sbi");
				}
				0x9B00..=0x9BFF => {
					instruction.push_str("sbis");
				}
				0x9C00..=0x9FFF => {
					instruction.push_str("mul");
					self.create_string_with_two_registers(0x9D, &mut operands);
				}
				0xA000..=0xA1FF => {
					instruction.push_str("ldd");
				}
				0xA200..=0xA3FF => {
					instruction.push_str("std");
				}
				0xA400..=0xA5FF => {
					instruction.push_str("ldd");
				}
				0xA600..=0xA7FF => {
					instruction.push_str("std");
				}
				0xA800..=0xA9FF => {
					instruction.push_str("ldd");
				}
				0xAA00..=0xABFF => {
					instruction.push_str("std");
				}
				0xAC00..=0xADFF => {
					instruction.push_str("ldd");
				}
				0xAE00..=0xAFFF => {
					instruction.push_str("std");
				}
				0xB000..=0xB7FF => {
					instruction.push_str("in_");
				}
				0xB800..=0xBFFF => {
					let source = (self.opcode & 0x1F0) >> 4;
					let a = (self.opcode & 0xF) | ((self.opcode & 0x600) >> 5);
					instruction.push_str("out");
					operands.push_str(format!("0x{:02X} [{}], r{}", a, a, source).as_str());
				}
				0xC000..=0xCFFF => {
					instruction.push_str("rjmp");
				}
				0xD000..=0xDFFF => {
					instruction.push_str("rcall");
				}
				0xE000..=0xEFFF => {
					instruction.push_str("ldi");
					self.create_string_with_register_and_constant(&mut operands);
				}
				0xF000..=0xF3FF => match low_byte {
					0x0 => {
						instruction.push_str("brcs");
					}
					0x1 => {
						instruction.push_str("breq");
					}
					0x2 => {
						instruction.push_str("brmi");
					}
					0x3 => {
						instruction.push_str("brvs");
					}
					0x4 => {
						instruction.push_str("brlt");
					}
					0x5 => {
						instruction.push_str("brhs");
					}
					0x6 => {
						instruction.push_str("brts");
					}
					0x7 => {
						instruction.push_str("brie");
					}
					0x8 => {
						instruction.push_str("brcs");
					}
					0x9 => {
						instruction.push_str("breq");
					}
					0xA => {
						instruction.push_str("brmi");
					}
					0xB => {
						instruction.push_str("brvs");
					}
					0xC => {
						instruction.push_str("brlt");
					}
					0xD => {
						instruction.push_str("brhs");
					}
					0xE => {
						instruction.push_str("brts");
					}
					0xF => {
						instruction.push_str("brie");
					}
					_ => unreachable!(),
				},
				0xF400..=0xF7FF => match low_byte {
					0x0 => {
						instruction.push_str("brcc");
					}
					0x1 => {
						instruction.push_str("brne");
					}
					0x2 => {
						instruction.push_str("brpl");
					}
					0x3 => {
						instruction.push_str("brvc");
					}
					0x4 => {
						instruction.push_str("brge");
					}
					0x5 => {
						instruction.push_str("brhc");
					}
					0x6 => {
						instruction.push_str("brtc");
					}
					0x7 => {
						instruction.push_str("brid");
					}
					0x8 => {
						instruction.push_str("brcc");
					}
					0x9 => {
						instruction.push_str("brne");
					}
					0xA => {
						instruction.push_str("brpl");
					}
					0xB => {
						instruction.push_str("brvc");
					}
					0xC => {
						instruction.push_str("brge");
					}
					0xD => {
						instruction.push_str("brhc");
					}
					0xE => {
						instruction.push_str("brtc");
					}
					0xF => {
						instruction.push_str("brid");
					}
					_ => unreachable!(),
				},
				0xF800..=0xF9FF => {
					instruction.push_str("bld");
				}
				0xFA00..=0xFBFF => {
					instruction.push_str("bst");
				}
				0xFC00..=0xFDFF => {
					instruction.push_str("sbrc");
				}
				0xFE00..=0xFFFF => {
					instruction.push_str("sbrs");
				}
			}
			assembly.insert(
				id,
				Instruction {
					address,
					opcode,
					instruction,
					operands,
				},
			);
			current_address += 1;
		}
		self.assembly = Some(assembly);
	}
}
