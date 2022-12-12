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

		self.sram.registers[d as usize] = result_low;
		self.sram.registers[(d + 1) as usize] = result_high;

		let r_bits = bits_u16(result);
		let rdh_bits = bits_u8(result_high);

		self.status.V = !rdh_bits.7 & r_bits.15 == 1;
		self.status.N = r_bits.15 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) == 1;
		self.status.Z = result == 0;
		self.status.C = !r_bits.15 & rdh_bits.7 == 1;

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

		let result = self.sram.registers[rd as usize] - self.sram.registers[rr as usize];

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

	fn sbiw(&mut self) {}

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

	fn andi(&mut self) {}

	fn or(&mut self) {}

	fn ori(&mut self) {}

	fn eor(&mut self) {}

	fn com(&mut self) {}

	fn neg(&mut self) {}

	fn sbr(&mut self) {}

	fn cbr(&mut self) {}

	fn inc(&mut self) {}

	fn dec(&mut self) {}

	fn tst(&mut self) {}

	fn clr(&mut self) {}

	fn ser(&mut self) {}

	fn mul(&mut self) {}

	fn muls(&mut self) {}

	fn mulsu(&mut self) {}

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

	fn brbs(&mut self) {}

	fn brbc(&mut self) {}

	fn breq(&mut self) {}

	fn brne(&mut self) {}

	fn brcs(&mut self) {}

	fn brcc(&mut self) {}

	fn brsh(&mut self) {}

	fn brlo(&mut self) {}

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

	fn lsl(&mut self) {}

	fn lsr(&mut self) {}

	fn rol(&mut self) {}

	fn ror(&mut self) {}

	fn asr(&mut self) {}

	fn swap(&mut self) {}

	fn bset(&mut self) {}

	fn bclr(&mut self) {}

	fn bst(&mut self) {}

	fn bld(&mut self) {}

	fn sec(&mut self) {}

	fn clc(&mut self) {}

	fn sen(&mut self) {}

	fn cln(&mut self) {}

	fn sez(&mut self) {}

	fn clz(&mut self) {}

	fn sei(&mut self) {}

	fn cli(&mut self) {}

	fn ses(&mut self) {}

	fn cls(&mut self) {}

	fn sev(&mut self) {}

	fn clv(&mut self) {}

	fn set(&mut self) {}

	fn clt(&mut self) {}

	fn seh(&mut self) {}

	fn clh(&mut self) {}

	// Data Transfer Instructions

	fn mov(&mut self) {}

	fn movw(&mut self) {}

	fn ldi(&mut self) {}

	fn ld(&mut self) {}

	fn ldd(&mut self) {}

	fn lds(&mut self) {}

	fn st(&mut self) {}

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

	pub fn step(&mut self) {
		self.opcode = self.system.program_memory.read(self.pc);

		match self.opcode {
			0x0000..=0x00FF => match (self.opcode & 0xFF) as u8 {
				0x00 => self.nop(),
				_ => panic!("Reserved opcode: {:x?}", self.opcode),
			},
			0x0100..=0x01FF => self.movw(),
			0x0200..=0x02FF => self.muls(),
			// mulsu, fmul, fmuls, fmulsu
			0x0300..=0x03FF => {}
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
			// lds, ld, lpm, elpm, pop
			0x9000..=0x91FF => {
				//
			}
			// sts, st, push
			0x9200..=0x93FF => {
				//
			}
			// asr, call, clc, clh, cli, cln, cls, clt, clv, com, dec, des, eijmp, ijmp
			// inc, jmp, lsr, neg, ror, sec, seh, sei, sen, ses, set, sev, sez, swap
			0x9400..=0x94FF => {}

			// asr, break, call, com, dec, eicall, elpm, icall, inc, jmp, lpm, lsr, neg
			// ret, reti, ror, sleep, spm, swap, wdr
			0x9500..=0x95FF => {
				//
			}
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
			// brcs, breq, brhs, brie, brlt, brmi, brts, brvs
			0xF000..=0xF3FF => {
				//
			}
			// brcc, brge, brhc, brid, brne, brpl, brtc, brvc
			0xF400..=0xF7FF => {
				//
			}
			0xF800..=0xF9FF => self.bld(),
			0xFA00..=0xFBFF => self.bst(),
			0xFC00..=0xFDFF => self.sbrc(),
			0xFE00..=0xFFFF => self.sbrs(),
		}
	}
}
