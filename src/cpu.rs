use crate::memory::{Memory, Sram};
use crate::system::System;
use crate::utils::{bits_u16, bits_u8, high_byte};

#[derive(Default, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Sreg {
	/// Carry Flag (Bit 0)
	pub C: bool,
	/// Zero Flag (Bit 1)
	pub Z: bool,
	/// Negative Flag (Bit 2)
	pub N: bool,
	/// Two's Compliment Overflow Flag (Bit 3)
	pub V: bool,
	/// Sign Bit (Bit 4)
	pub S: bool,
	/// Half Carry Flag (Bit 5)
	pub H: bool,
	/// Bit Copy Storage (Bit 6)
	pub T: bool,
	/// Global Interrupt Enable (Bit 7)
	pub I: bool,
}

#[allow(dead_code)]
impl Sreg {
	pub fn byte(&self) -> u8 {
		(self.C as u8)
			| (self.Z as u8) << 1
			| (self.N as u8) << 2
			| (self.V as u8) << 3
			| (self.S as u8) << 4
			| (self.H as u8) << 5
			| (self.T as u8) << 6
			| (self.I as u8) << 7
	}

	pub fn set_byte(&mut self, byte: u8) {
		self.C = ((byte) & 1) != 0;
		self.Z = ((byte >> 1) & 1) != 0;
		self.N = ((byte >> 2) & 1) != 0;
		self.V = ((byte >> 3) & 1) != 0;
		self.S = ((byte >> 4) & 1) != 0;
		self.H = ((byte >> 5) & 1) != 0;
		self.T = ((byte >> 6) & 1) != 0;
		self.I = ((byte >> 7) & 1) != 0;
	}
}

#[derive(Default)]
pub struct Cpu {
	pub system: System,
	pub sram: Sram,
	pub sp: u16,
	pub status: Sreg,
	pub pc: u16,
	pub cycles: usize,
	pub opcode: u16,
}

impl Cpu {
	pub fn init() -> Self {
		Self {
			system: System::default(),
			sram: Sram::default(),
			sp: 0x0000,
			status: Sreg::default(),
			pc: 0x0000,
			cycles: 0,
			opcode: 0x0000,
		}
	}

	pub fn reset(&mut self) {
		self.sp = 0x0000;
		self.pc = 0x0000;
		self.cycles = 0;
	}

	// Arithmetic and Logic Instruction

	fn add(&mut self) {
		// 0000 11rd dddd rrrr

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		let mut rr = (self.opcode & 0xF) as u8;

		match high_byte(self.opcode) {
			0x0D => rd += 16,
			0x0E => rr += 16,
			0x0F => {
				rd += 16;
				rr += 16;
			}
			_ => {}
		}

		let result = self.sram.registers[rd as usize] + self.sram.registers[rr as usize];

		let r_bits = bits_u8(result);
		let rd_bits = bits_u8(rd);
		let rr_bits = bits_u8(rr);

		self.status.H =
			(rd_bits.3 & rr_bits.3 | rr_bits.3 & !r_bits.3 | !r_bits.3 & rd_bits.3) == 1;
		self.status.V =
			(rd_bits.7 & rr_bits.7 & !r_bits.7 | !rd_bits.7 & !rr_bits.7 & r_bits.7) == 1;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;
		self.status.C =
			(rd_bits.7 & rr_bits.7 | rr_bits.7 & !r_bits.7 | !r_bits.7 & rd_bits.7) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn adc(&mut self) {
		// 0001 11rd dddd rrrr

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		let mut rr = (self.opcode & 0xF) as u8;

		match high_byte(self.opcode) {
			0x1D => rd += 16,
			0x1E => rr += 16,
			0x1F => {
				rd += 16;
				rr += 16;
			}
			_ => {}
		}

		let result = self.sram.registers[rd as usize]
			+ self.sram.registers[rr as usize]
			+ self.status.C as u8;

		let r_bits = bits_u8(result);
		let rd_bits = bits_u8(rd);
		let rr_bits = bits_u8(rr);

		self.status.H = (rd_bits.3 & rr_bits.3 | rr_bits.3 & !r_bits.3 & r_bits.3 & rd_bits.3) == 1;
		self.status.V =
			(rd_bits.7 & rr_bits.7 & !r_bits.7 | !rd_bits.7 & !rr_bits.7 & r_bits.7) == 1;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;
		self.status.C =
			(rd_bits.7 & rr_bits.7 | rr_bits.7 & !r_bits.7 | !r_bits.7 & rd_bits.7) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn adiw(&mut self) {
		// 1001 0110 KKdd KKKK

		let mut k = self.opcode & 0xF;
		k |= (self.opcode & (1 << 6)) >> 2;
		k |= (self.opcode & (1 << 7)) >> 2;

		let d = (((self.opcode >> 4 & 0xF) & 0x3) * 2 + 24) as u8;

		let rd_low = self.sram.registers[d as usize] as u16;
		let rd_high = self.sram.registers[(d + 1) as usize] as u16;
		let rd = (rd_high << 8) | rd_low;

		let result = rd + k;
		let result_low = (result & 0xFF) as u8;
		let result_high = ((result >> 8) & 0xFF) as u8;

		let r_bits = bits_u16(result);
		let rdh_bits = bits_u8(result_high);

		self.status.V = (!rdh_bits.7 & r_bits.15) == 1;
		self.status.N = r_bits.15 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;
		self.status.C = (!r_bits.15 & rdh_bits.7) == 1;

		self.sram.registers[d as usize] = result_low;
		self.sram.registers[(d + 1) as usize] = result_high;

		self.pc += 1;
		self.cycles += 2;
	}

	fn sub(&mut self) {
		// 0001 10rd dddd rrrr

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		let mut rr = (self.opcode & 0xF) as u8;

		match high_byte(self.opcode) {
			0x19 => rd += 16,
			0x1A => rr += 16,
			0x1B => {
				rd += 16;
				rr += 16;
			}
			_ => {}
		}

		let result =
			self.sram.registers[rd as usize].wrapping_sub(self.sram.registers[rr as usize]);

		let r_bits = bits_u8(result);
		let rd_bits = bits_u8(rd);
		let rr_bits = bits_u8(rr);

		self.status.H =
			(!rd_bits.3 & rr_bits.3 | rr_bits.3 & r_bits.3 | r_bits.3 & !rd_bits.3) == 1;
		self.status.V =
			(rd_bits.7 & !rr_bits.7 & !r_bits.7 | !rd_bits.7 & rr_bits.7 & r_bits.7) == 1;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;
		self.status.C =
			(!rd_bits.7 & rr_bits.7 | rr_bits.7 & r_bits.7 | r_bits.7 & !rd_bits.7) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn subi(&mut self) {
		// 0101 KKKK dddd KKKK

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		rd += 16;

		let k = ((((self.opcode >> 8) & 0xF) << 4) | (self.opcode & 0xF)) as u8;
		let result = self.sram.registers[rd as usize] - k;

		let r_bits = bits_u8(result);
		let rd_bits = bits_u8(rd);
		let k_bits = bits_u8(k);

		self.status.H = (!rd_bits.3 & k_bits.3 | k_bits.3 & r_bits.3 | r_bits.3 & !rd_bits.3) == 1;
		self.status.V = (rd_bits.7 & !k_bits.7 & !r_bits.7 | !rd_bits.7 & k_bits.7 & r_bits.7) == 1;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;
		self.status.C = (!rd_bits.7 & k_bits.7 | k_bits.7 & r_bits.7 | r_bits.7 & !rd_bits.7) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn sbc(&mut self) {
		// 0000 10rd dddd rrrr

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		let mut rr = (self.opcode & 0xF) as u8;

		match high_byte(self.opcode) {
			0x09 => rd += 16,
			0x0A => rr += 16,
			0x0B => {
				rd += 16;
				rr += 16;
			}
			_ => {}
		}

		let result = self.sram.registers[rd as usize]
			- self.sram.registers[rr as usize]
			- self.status.C as u8;

		let r_bits = bits_u8(result);
		let rd_bits = bits_u8(rd);
		let rr_bits = bits_u8(rr);

		self.status.H =
			(!rd_bits.3 & rr_bits.3 | rr_bits.3 & r_bits.3 | r_bits.3 & !rd_bits.3) == 1;
		self.status.V =
			(rd_bits.7 & !rr_bits.7 & !r_bits.7 | !rd_bits.7 & rr_bits.7 & r_bits.7) == 1;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;
		self.status.C =
			(!rd_bits.7 & rr_bits.7 | rr_bits.7 & r_bits.7 | r_bits.7 & !rd_bits.7) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn sbci(&mut self) {
		// 0100 KKKK dddd KKKK

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		rd += 16;

		let k = ((((self.opcode >> 8) & 0xF) << 4) | (self.opcode & 0xF)) as u8;
		let result = self.sram.registers[rd as usize] - k - self.status.C as u8;

		let r_bits = bits_u8(result);
		let rd_bits = bits_u8(rd);
		let k_bits = bits_u8(k);

		self.status.H = (!rd_bits.3 & k_bits.3 | k_bits.3 & r_bits.3 | r_bits.3 & !rd_bits.3) == 1;
		self.status.V = (rd_bits.7 & !k_bits.7 & !r_bits.7 | !rd_bits.7 & k_bits.7 & r_bits.7) == 1;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;
		self.status.C = (!rd_bits.7 & k_bits.7 | k_bits.7 & r_bits.7 | r_bits.7 & !rd_bits.7) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn sbiw(&mut self) {
		// 1001 0111 KKdd KKKK

		let mut k = self.opcode & 0xF;
		k |= (self.opcode & (1 << 6)) >> 2;
		k |= (self.opcode & (1 << 7)) >> 2;

		let d = (((self.opcode >> 4 & 0xF) & 0x3) * 2 + 24) as u8;

		let rd_low = self.sram.registers[d as usize] as u16;
		let rd_high = self.sram.registers[(d + 1) as usize] as u16;
		let rd = (rd_high << 8) | rd_low;

		let result = rd - k;
		let result_low = (result & 0xFF) as u8;
		let result_high = ((result >> 8) & 0xFF) as u8;

		let r_bits = bits_u16(result);
		let rdh_bits = bits_u8(result_high);

		// set flags
		self.status.V = (r_bits.15 & !rdh_bits.7) == 1;
		self.status.N = r_bits.15 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;
		self.status.C = r_bits.15 & !rdh_bits.7 == 1;

		self.sram.registers[d as usize] = result_low;
		self.sram.registers[(d + 1) as usize] = result_high;

		self.pc += 1;
		self.cycles += 2;
	}

	fn and(&mut self) {
		// 0010 00rd dddd rrrr

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		let mut rr = (self.opcode & 0xF) as u8;

		match high_byte(self.opcode) {
			0x21 => rd += 16,
			0x22 => rr += 16,
			0x23 => {
				rd += 16;
				rr += 16;
			}
			_ => {}
		}

		let result = self.sram.registers[rd as usize] & self.sram.registers[rr as usize];

		let r = bits_u8(result);

		self.status.V = false;
		self.status.N = r.7 == 1;
		self.status.Z = result == 0;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn andi(&mut self) {
		// 0111 KKKK dddd KKKK

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		rd += 16;

		let k = ((((self.opcode >> 8) & 0xF) << 4) | (self.opcode & 0xF)) as u8;
		let result = self.sram.registers[rd as usize] & k;

		let r_bits = bits_u8(result);

		self.status.V = false;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn or(&mut self) {
		// 0010 10rd dddd rrr

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		let mut rr = (self.opcode & 0xF) as u8;

		match high_byte(self.opcode) {
			0x29 => rd += 16,
			0x2A => rr += 16,
			0x2B => {
				rd += 16;
				rr += 16;
			}
			_ => {}
		}

		let result = self.sram.registers[rd as usize] | self.sram.registers[rr as usize];

		let r_bits = bits_u8(result);

		self.status.V = false;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn ori(&mut self) {
		// 0110 KKKK dddd KKKK

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		rd += 16;

		let k = ((((self.opcode >> 8) & 0xF) << 4) | (self.opcode & 0xF)) as u8;
		let result = self.sram.registers[rd as usize] | k;

		let r_bits = bits_u8(result);

		self.status.V = false;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn eor(&mut self) {
		// 0010 01rd dddd rrrr

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		let mut rr = (self.opcode & 0xF) as u8;

		match high_byte(self.opcode) {
			0x25 => rd += 16,
			0x26 => rr += 16,
			0x27 => {
				rd += 16;
				rr += 16;
			}
			_ => {}
		}

		let result = self.sram.registers[rd as usize] ^ self.sram.registers[rr as usize];

		let r_bits = bits_u8(result);

		self.status.V = false;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn com(&mut self) {
		// 1001 010d dddd 0000

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;

		if high_byte(self.opcode) == 0x95 {
			rd += 16
		}

		let result = 0xFF - self.sram.registers[rd as usize];
		let r_bits = bits_u8(result);

		self.status.V = false;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;
		self.status.C = true;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn neg(&mut self) {
		// 1001 010d dddd 0001

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;

		if high_byte(self.opcode) == 0x95 {
			rd += 16
		}

		let result = 0x00_u8.wrapping_sub(self.sram.registers[rd as usize]);
		let r_bits = bits_u8(result);
		let rd_bits = bits_u8(rd);

		self.status.H = (r_bits.3 | !rd_bits.3) == 1;
		self.status.V = (r_bits.7
			& !r_bits.6 & !r_bits.5
			& !r_bits.4 & !r_bits.3
			& !r_bits.2 & !r_bits.1
			& !r_bits.0) == 1;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;
		self.status.C =
			(r_bits.7 | r_bits.6 | r_bits.5 | r_bits.4 | r_bits.3 | r_bits.2 | r_bits.1 | r_bits.0)
				== 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	#[allow(dead_code)]
	fn sbr(&mut self) {
		// sbr is the same as ori
		// sbr r16, 0xFF -> ori r16, 0xFF
	}

	#[allow(dead_code)]
	fn cbr(&mut self) {
		// cbr is the same as andi but first negates the constant K
	}

	fn inc(&mut self) {
		// 1001 010d dddd 0011

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;

		if high_byte(self.opcode) == 0x95 {
			rd += 16
		}

		let result = self.sram.registers[rd as usize] + 1;
		let r_bits = bits_u8(result);

		self.status.V = (r_bits.7
			& !r_bits.6 & !r_bits.5
			& !r_bits.4 & !r_bits.3
			& !r_bits.2 & !r_bits.1
			& !r_bits.0) == 1;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = (!r_bits.7
			& !r_bits.6 & !r_bits.5
			& !r_bits.4 & !r_bits.3
			& !r_bits.2 & !r_bits.1
			& !r_bits.0) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn dec(&mut self) {
		// 1001 010d dddd 1010

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;

		if high_byte(self.opcode) == 0x95 {
			rd += 16
		}

		let result = self.sram.registers[rd as usize].wrapping_sub(0x01);
		let r_bits = bits_u8(result);

		self.status.V = (!r_bits.7
			& r_bits.6 & r_bits.5
			& r_bits.4 & r_bits.3
			& r_bits.2 & r_bits.1
			& r_bits.0) == 1;
		self.status.N = r_bits.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = (!r_bits.7
			& !r_bits.6 & !r_bits.5
			& !r_bits.4 & !r_bits.3
			& !r_bits.2 & !r_bits.1
			& !r_bits.0) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn des(&mut self) {
		// 1001 0100 KKKK 1011

		let _k = ((self.opcode & 0xF0) >> 4) as u8;

		// if the DES instruction is succeeding a non-DES instruction, an extra cycle is inserted.

		self.pc += 1;
		self.cycles += 1;
	}

	#[allow(dead_code)]
	fn tst(&mut self) {
		// tst is the same as and with the same register
		// as the source and destination
		// tst r0 -> and r0, r0
	}

	#[allow(dead_code)]
	fn clr(&mut self) {
		// clr is the same as eor with the same register
		// as the source and destination
		// clr r16 -> eor r16, r16
	}

	#[allow(dead_code)]
	fn ser(&mut self) {
		// ser is the same as ldi with 0xFF as the constant K
		// ser r16 -> ldi r16, 0xFF
	}

	fn mul(&mut self) {
		// 1001 11rd dddd rrrr

		let mut rd = ((self.opcode & 0xF0) >> 4) as u8;
		let mut rr = (self.opcode & 0xF) as u8;

		match high_byte(self.opcode) {
			0x9D => rd += 16,
			0x9E => rr += 16,
			0x9F => {
				rd += 16;
				rr += 16;
			}
			_ => {}
		}

		let result =
			(self.sram.registers[rd as usize] as u16) * (self.sram.registers[rr as usize] as u16);

		let result_low = (result & 0xFF) as u8;
		let result_high = ((result >> 8) & 0xFF) as u8;

		let r_bits = bits_u16(result);

		self.status.C = r_bits.15 == 1;
		self.status.Z = result == 1;

		self.sram.registers[0] = result_low;
		self.sram.registers[1] = result_high;

		self.pc += 1;
		self.cycles += 1;
	}

	fn muls(&mut self) {
		// 0000 0010 dddd rrrr

		let rd = (((self.opcode & 0xF0) >> 4) as u8) + 16;
		let rr = ((self.opcode & 0xF) as u8) + 16;

		let result = (self.sram.registers[rd as usize].wrapping_neg() as u16)
			* (self.sram.registers[rr as usize].wrapping_neg() as u16);

		let result_low = (result & 0xFF) as u8;
		let result_high = ((result >> 8) & 0xFF) as u8;

		let r_bits = bits_u16(result);

		self.status.C = r_bits.15 == 1;
		self.status.Z = result == 1;

		self.sram.registers[0] = result_low;
		self.sram.registers[1] = result_high;

		self.pc += 1;
		self.cycles += 1;
	}

	fn mulsu(&mut self) {
		// 0000 0011 0ddd 0rrr

		let rd = (((self.opcode & 0x70) >> 4) as u8) + 16;
		let rr = ((self.opcode & 0x7) as u8) + 16;

		let result = (self.sram.registers[rd as usize] as u16)
			.wrapping_neg()
			.wrapping_mul(self.sram.registers[rr as usize] as u16);

		let result_low = (result & 0xFF) as u8;
		let result_high = ((result >> 8) & 0xFF) as u8;

		let r_bits = bits_u16(result);

		self.status.C = r_bits.15 == 1;
		self.status.Z = result == 1;

		self.sram.registers[0] = result_low;
		self.sram.registers[1] = result_high;

		self.pc += 1;
		self.cycles += 1;
	}

	fn fmul(&mut self) {}

	fn fmuls(&mut self) {}

	fn fmulsu(&mut self) {}

	// Branch Instructions

	fn rjmp(&mut self) {}

	fn ijmp(&mut self) {}

	fn jmp(&mut self) {}

	fn rcall(&mut self) {}

	fn icall(&mut self) {}

	fn call(&mut self) {}

	fn ret(&mut self) {}

	fn reti(&mut self) {}

	fn cpse(&mut self) {}

	fn cp(&mut self) {}

	fn cpc(&mut self) {}

	fn cpi(&mut self) {}

	fn sbrc(&mut self) {}

	fn sbrs(&mut self) {}

	fn sbic(&mut self) {}

	fn sbis(&mut self) {}

	#[allow(dead_code)]
	fn brbs(&mut self) {
		// brbs 0, <label> -> brcs <address>
		// brbs 1, <label> -> breq <address>
		// brbs 2, <label> -> brmi <address>
		// brbs 3, <label> -> brvs <address>
		// brbs 4, <label> -> brlt <address>
		// brbs 5, <label> -> brhs <address>
		// brbs 6, <label> -> brts <address>
		// brbs 7, <label> -> brie <address>
	}

	#[allow(dead_code)]
	fn brbc(&mut self) {
		// brbc 0, <label> -> brcc <address>
		// brbc 1, <label> -> brne <address>
		// brbc 2, <label> -> brpl <address>
		// brbc 3, <label> -> brvc <address>
		// brbc 4, <label> -> brge <address>
		// brbc 5, <label> -> brhc <address>
		// brbc 6, <label> -> brtc <address>
		// brbc 7, <label> -> brid <address>
	}

	fn breq(&mut self) {}

	fn brne(&mut self) {}

	fn brcs(&mut self) {}

	fn brcc(&mut self) {}

	#[allow(dead_code)]
	fn brsh(&mut self) {
		// brsh <label> -> brcc <address>
	}

	#[allow(dead_code)]
	fn brlo(&mut self) {
		// brlo <label> -> brbs 0, <label> -> brcs <address>
	}

	fn brmi(&mut self) {}

	fn brpl(&mut self) {}

	fn brge(&mut self) {}

	fn brlt(&mut self) {}

	fn brhs(&mut self) {}

	fn brhc(&mut self) {}

	fn brts(&mut self) {}

	fn brtc(&mut self) {}

	fn brvs(&mut self) {}

	fn brvc(&mut self) {}

	fn brie(&mut self) {}

	fn brid(&mut self) {}

	// Bit and Bit-Test Instructions

	fn sbi(&mut self) {}

	fn cbi(&mut self) {}

	#[allow(dead_code)]
	fn lsl(&mut self) {
		// lsl r0 -> add r0, r0
		// lsl r1 -> add r1, r1
		// lsl r5 -> add r5, r5
	}

	fn lsr(&mut self) {}

	#[allow(dead_code)]
	fn rol(&mut self) {
		// rol r0 -> adc r0, r0
		// rol r5 -> adc r5, r5
	}

	fn ror(&mut self) {}

	fn asr(&mut self) {}

	fn swap(&mut self) {}

	#[allow(dead_code)]
	fn bset(&mut self) {
		// bset 0 -> sec
		// bset 1 -> sez
		// bset 2 -> sen
		// bset 3 -> sev
		// bset 4 -> ses
		// bset 5 -> seh
		// bset 6 -> set
		// bset 7 -> sei
	}

	#[allow(dead_code)]
	fn bclr(&mut self) {
		// bclr 0 -> clc
		// bclr 1 -> clz
		// bclr 2 -> cln
		// bclr 3 -> clv
		// bclr 4 -> cls
		// bclr 5 -> clh
		// bclr 6 -> clt
		// bclr 7 -> cli
	}

	fn bst(&mut self) {}

	fn bld(&mut self) {}

	fn sec(&mut self) {
		self.status.C = true;
		self.pc += 1;
		self.cycles += 1;
	}

	fn clc(&mut self) {
		self.status.C = false;
		self.pc += 1;
		self.cycles += 1;
	}

	fn sen(&mut self) {
		self.status.N = true;
		self.pc += 1;
		self.cycles += 1;
	}

	fn cln(&mut self) {
		self.status.N = false;
		self.pc += 1;
		self.cycles += 1;
	}

	fn sez(&mut self) {
		self.status.Z = true;
		self.pc += 1;
		self.cycles += 1;
	}

	fn clz(&mut self) {
		self.status.Z = false;
		self.pc += 1;
		self.cycles += 1;
	}

	fn sei(&mut self) {
		self.status.I = true;
		self.pc += 1;
		self.cycles += 1;
	}

	fn cli(&mut self) {
		self.status.I = false;
		self.pc += 1;
		self.cycles += 1;
	}

	fn ses(&mut self) {
		self.status.S = true;
		self.pc += 1;
		self.cycles += 1;
	}

	fn cls(&mut self) {
		self.status.S = false;
		self.pc += 1;
		self.cycles += 1;
	}

	fn sev(&mut self) {
		self.status.V = true;
		self.pc += 1;
		self.cycles += 1;
	}

	fn clv(&mut self) {
		self.status.V = false;
		self.pc += 1;
		self.cycles += 1;
	}

	fn set(&mut self) {
		self.status.T = true;
		self.pc += 1;
		self.cycles += 1;
	}

	fn clt(&mut self) {
		self.status.T = false;
		self.pc += 1;
		self.cycles += 1;
	}

	fn seh(&mut self) {
		self.status.H = true;
		self.pc += 1;
		self.cycles += 1;
	}

	fn clh(&mut self) {
		self.status.H = false;
		self.pc += 1;
		self.cycles += 1;
	}

	// Data Transfer Instructions

	fn mov(&mut self) {}

	fn movw(&mut self) {}

	fn ldi(&mut self) {}

	fn ld_x(&mut self) {}

	fn ld_y(&mut self) {}

	fn ld_z(&mut self) {}

	fn ldd(&mut self) {}

	fn lds(&mut self) {}

	fn st_x(&mut self) {}

	fn st_y(&mut self) {}

	fn st_z(&mut self) {}

	fn std(&mut self) {}

	fn sts(&mut self) {}

	fn lpm(&mut self) {}

	fn spm(&mut self) {}

	fn in_(&mut self) {}

	fn out(&mut self) {}

	fn push(&mut self) {}

	fn pop(&mut self) {}

	// MCU Control Instructions

	fn nop(&mut self) {
		self.cycles += 1;
		self.pc += 1;
	}

	fn sleep(&mut self) {
		self.cycles += 1;
		self.pc += 1;
	}

	fn wdr(&mut self) {
		self.cycles += 1;
		self.pc += 1;
	}

	fn break_(&mut self) {}

	fn reserved(&mut self) {
		println!("Reserved opcode: {:x?}", self.opcode);
		self.cycles += 1;
		self.pc += 1;
	}

	pub fn step(&mut self) {
		self.opcode = self.system.program_memory.read(self.pc);

		let low_byte = (self.opcode & 0xF) as u8;
		let high_byte = ((self.opcode >> 4) & 0xF) as u8;

		match self.opcode {
			0x0000..=0x00FF => match (self.opcode & 0xFF) as u8 {
				0x00 => self.nop(),
				_ => self.reserved(),
			},
			0x0100..=0x01FF => self.movw(),
			0x0200..=0x02FF => self.muls(),
			0x0300..=0x03FF => match low_byte {
				0x0..=0x7 => match high_byte {
					0x0..=0x7 => self.mulsu(),
					0x8..=0xF => self.fmuls(),
					_ => unreachable!(),
				},
				0x8..=0xF => match high_byte {
					0x0..=0x7 => self.fmul(),
					0x8..=0xF => self.fmulsu(),
					_ => unreachable!(),
				},
				_ => unreachable!(),
			},
			0x0400..=0x07FF => self.cpc(),
			0x0800..=0x0BFF => self.sbc(),
			0x0C00..=0x0FFF => self.add(),
			0x1000..=0x13FF => self.cpse(),
			0x1400..=0x17FF => self.cp(),
			0x1800..=0x1BFF => self.sub(),
			0x1C00..=0x1FFF => self.adc(),
			0x2000..=0x23FF => self.and(),
			0x2400..=0x27FF => self.eor(),
			0x2800..=0x2BFF => self.or(),
			0x2C00..=0x2FFF => self.mov(),
			0x3000..=0x3FFF => self.cpi(),
			0x4000..=0x4FFF => self.sbci(),
			0x5000..=0x5FFF => self.subi(),
			0x6000..=0x6FFF => self.ori(),
			0x7000..=0x7FFF => self.andi(),
			0x8000..=0x81FF => self.ldd(),
			0x8200..=0x83FF => self.std(),
			0x8400..=0x85FF => self.ldd(),
			0x8600..=0x87FF => self.std(),
			0x8800..=0x89FF => self.ldd(),
			0x8A00..=0x8BFF => self.std(),
			0x8C00..=0x8DFF => self.ldd(),
			0x8E00..=0x8FFF => self.std(),
			0x9000..=0x91FF => match low_byte {
				0x0 => self.lds(),
				0x1..=0x2 => self.ld_z(),
				0x3 => self.reserved(),
				0x4..=0x5 => self.lpm(),
				0x6..=0x8 => self.reserved(),
				0x9..=0xA => self.ld_y(),
				0xB => self.reserved(),
				0xC..=0xE => self.ld_x(),
				0xF => self.pop(),
				_ => unreachable!(),
			},
			0x9200..=0x93FF => match low_byte {
				0x0 => self.sts(),
				0x1..=0x2 => self.st_z(),
				0x3..=0x8 => self.reserved(),
				0x9..=0xA => self.st_y(),
				0xB => self.reserved(),
				0xC..=0xE => self.st_x(),
				0xF => self.push(),
				_ => unreachable!(),
			},
			0x9400..=0x94FF => match low_byte {
				0x0 => self.com(),
				0x1 => self.neg(),
				0x2 => self.swap(),
				0x3 => self.inc(),
				0x4 => self.reserved(),
				0x5 => self.asr(),
				0x6 => self.lsr(),
				0x7 => self.ror(),
				0x8 => match high_byte {
					0x0 => self.sec(),
					0x1 => self.sez(),
					0x2 => self.sen(),
					0x3 => self.sev(),
					0x4 => self.ses(),
					0x5 => self.seh(),
					0x6 => self.set(),
					0x7 => self.sei(),
					0x8 => self.clc(),
					0x9 => self.clz(),
					0xA => self.cln(),
					0xB => self.clv(),
					0xC => self.cls(),
					0xD => self.clh(),
					0xE => self.clt(),
					0xF => self.cli(),
					_ => unreachable!(),
				},
				0x9 => match high_byte {
					0x0 => self.ijmp(),
					_ => self.reserved(),
				},
				0xA => self.dec(),
				0xB => self.des(),
				0xC..=0xD => self.jmp(),
				0xE..=0xF => self.call(),
				_ => unreachable!(),
			},
			0x9500..=0x95FF => match low_byte {
				0x00 => self.com(),
				0x01 => self.neg(),
				0x02 => self.swap(),
				0x03 => self.inc(),
				0x04 => self.reserved(),
				0x05 => self.asr(),
				0x06 => self.lsr(),
				0x07 => self.ror(),
				0x08 => match high_byte {
					0x0 => self.ret(),
					0x1 => self.reti(),
					0x8 => self.sleep(),
					0x9 => self.break_(),
					0xA => self.wdr(),
					0xC => self.lpm(),
					0xE..=0xF => self.spm(),
					_ => self.reserved(),
				},
				0x09 => match high_byte {
					0x0 => self.icall(),
					_ => self.reserved(),
				},
				0x0A => self.dec(),
				0x0B => self.reserved(),
				0xC..=0xD => self.jmp(),
				0x0E..=0x0F => self.call(),
				_ => unreachable!(),
			},
			0x9600..=0x96FF => self.adiw(),
			0x9700..=0x97FF => self.sbiw(),
			0x9800..=0x98FF => self.cbi(),
			0x9900..=0x99FF => self.sbic(),
			0x9A00..=0x9AFF => self.sbi(),
			0x9B00..=0x9BFF => self.sbis(),
			0x9C00..=0x9FFF => self.mul(),
			0xA000..=0xA1FF => self.ldd(),
			0xA200..=0xA3FF => self.std(),
			0xA400..=0xA5FF => self.ldd(),
			0xA600..=0xA7FF => self.std(),
			0xA800..=0xA9FF => self.ldd(),
			0xAA00..=0xABFF => self.std(),
			0xAC00..=0xADFF => self.ldd(),
			0xAE00..=0xAFFF => self.std(),
			0xB000..=0xB7FF => self.in_(),
			0xB800..=0xBFFF => self.out(),
			0xC000..=0xCFFF => self.rjmp(),
			0xD000..=0xDFFF => self.rcall(),
			0xE000..=0xEFFF => self.ldi(),
			0xF000..=0xF3FF => match low_byte {
				0x0 => self.brcs(),
				0x1 => self.breq(),
				0x2 => self.brmi(),
				0x3 => self.brvs(),
				0x4 => self.brlt(),
				0x5 => self.brhs(),
				0x6 => self.brts(),
				0x7 => self.brie(),
				0x8 => self.brcs(),
				0x9 => self.breq(),
				0xA => self.brmi(),
				0xB => self.brvs(),
				0xC => self.brlt(),
				0xD => self.brhs(),
				0xE => self.brts(),
				0xF => self.brie(),
				_ => unreachable!(),
			},
			0xF400..=0xF7FF => match low_byte {
				0x0 => self.brcc(),
				0x1 => self.brne(),
				0x2 => self.brpl(),
				0x3 => self.brvc(),
				0x4 => self.brge(),
				0x5 => self.brhc(),
				0x6 => self.brtc(),
				0x7 => self.brid(),
				0x8 => self.brcc(),
				0x9 => self.brne(),
				0xA => self.brpl(),
				0xB => self.brvc(),
				0xC => self.brge(),
				0xD => self.brhc(),
				0xE => self.brtc(),
				0xF => self.brid(),
				_ => unreachable!(),
			},
			0xF800..=0xF9FF => self.bld(),
			0xFA00..=0xFBFF => self.bst(),
			0xFC00..=0xFDFF => self.sbrc(),
			0xFE00..=0xFFFF => self.sbrs(),
		}
	}
}
