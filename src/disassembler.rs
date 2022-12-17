use crate::memory::{ApplicationFlash, Memory};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub struct InstructionString {
	pub address: String,
	pub opcode: String,
	pub instruction: String,
}

#[derive(Debug)]
pub struct Disassembler {
	pub assembly: Option<BTreeMap<u16, InstructionString>>,
}

impl Default for Disassembler {
	fn default() -> Self {
		Self { assembly: None }
	}
}

impl Disassembler {
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
			let address = format!("0x{:04X}", current_address);
			let opcode = program.read(current_address);
			let opcode_string = format!("0x{:04X}", opcode);

			let low_byte = (opcode & 0xF) as u8;
			let high_byte = ((opcode >> 4) & 0xF) as u8;

			let mut instruction = String::new();

			match opcode {
				0x0000..=0x00FF => match (opcode & 0xFF) as u8 {
					0x00 => instruction.push_str("nop"),
					_ => instruction.push_str("[R]"),
				},
				0x0100..=0x01FF => {
					//movw
				}
				0x0200..=0x02FF => {
					//muls
				}
				0x0300..=0x03FF => match low_byte {
					0x0..=0x7 => match high_byte {
						0x0..=0x7 => {
							//mulsu
						}
						0x8..=0xF => {
							//fmuls
						}
						_ => unreachable!(),
					},
					0x8..=0xF => match high_byte {
						0x0..=0x7 => {
							//fmul
						}
						0x8..=0xF => {
							//fmulsu
						}
						_ => unreachable!(),
					},
					_ => unreachable!(),
				},
				0x0400..=0x07FF => {
					//cpc
				}
				0x0800..=0x0BFF => {
					//sbc
				}
				0x0C00..=0x0FFF => {
					//add
				}
				0x1000..=0x13FF => {
					//cpse
				}
				0x1400..=0x17FF => {
					//cp
				}
				0x1800..=0x1BFF => {
					//sub
				}
				0x1C00..=0x1FFF => {
					//adc
				}
				0x2000..=0x23FF => {
					//and
				}
				0x2400..=0x27FF => {
					//eor
				}
				0x2800..=0x2BFF => {
					//or
				}
				0x2C00..=0x2FFF => {
					//mov
				}
				0x3000..=0x3FFF => {
					//cpi
				}
				0x4000..=0x4FFF => {
					//sbci
				}
				0x5000..=0x5FFF => {
					//subi
				}
				0x6000..=0x6FFF => {
					//ori
				}
				0x7000..=0x7FFF => {
					//andi
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
						//com
					}
					0x1 => {
						//neg
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
						//dec
					}
					0xB => {
						//des
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
						//com
					}
					0x01 => {
						//neg
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
						//dec
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
					//adiw
				}
				0x9700..=0x97FF => {
					//sbiw
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
					//mul
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
					opcode: opcode_string,
					instruction,
				},
			);
			current_address += 1;
		}
		self.assembly = Some(assembly);
	}
}
