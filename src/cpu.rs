use crate::memory::{Memory, Sram};
use crate::system::System;
use crate::utils::{all_bits_u16, all_bits_u8, high_byte};

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
}

impl Cpu {
	pub fn init(system: System) -> Self {
		Self {
			system,
			sram: Sram::default(),
			sp: 0x00,
			status: Sreg::default(),
			pc: 0x0000,
			cycles: 0,
		}
	}

	pub fn reset(&mut self) {}

	pub fn add(&mut self, opcode: u16) {
		// 0000 11rd dddd rrrr

		let mut rd = ((opcode & 0xF0) >> 4) as u8;
		let mut rr = (opcode & 0xF) as u8;

		match high_byte(opcode) {
			0x0D => rd += 16,
			0x0E => rr += 16,
			0x0F => {
				rd += 16;
				rr += 16;
			}
			_ => {}
		}

		let result = self.sram.registers[rd as usize] + self.sram.registers[rr as usize];

		let r = all_bits_u8(result);
		let rd_bits = all_bits_u8(rd);
		let rr_bits = all_bits_u8(rr);

		self.status.H = (rd_bits.3 & rr_bits.3 | rr_bits.3 & !r.3 | !r.3 & rd_bits.3) == 1;
		self.status.V = (rd_bits.7 & rr_bits.7 & !r.7 | !rd_bits.7 & !rr_bits.7 & r.7) == 1;
		self.status.N = r.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) != 0;
		self.status.Z = result == 0;
		self.status.C = (rd_bits.7 & rr_bits.7 | rr_bits.7 & !r.7 | !r.7 & rd_bits.7) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn adc(&mut self, opcode: u16) {
		// 0001 11rd dddd rrrr

		let mut rd = ((opcode & 0xF0) >> 4) as u8;
		let mut rr = (opcode & 0xF) as u8;

		match high_byte(opcode) {
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

		let r = all_bits_u8(result);
		let rd_bits = all_bits_u8(rd);
		let rr_bits = all_bits_u8(rr);

		self.status.H = (rd_bits.3 & rr_bits.3 | rr_bits.3 & !r.3 & r.3 & rd_bits.3) == 1;
		self.status.V = (rd_bits.7 & rr_bits.7 & !r.7 | !rd_bits.7 & !rr_bits.7 & r.7) == 1;
		self.status.N = r.7 == 1;
		self.status.S = ((self.status.N as u8) ^ (self.status.V as u8)) != 0;
		self.status.Z = result == 0;
		self.status.C = (rd_bits.7 & rr_bits.7 | rr_bits.7 & !r.7 | !r.7 & rd_bits.7) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn adiw(&mut self, opcode: u16) {
		// 1001 0110 KKdd KKKK

		let mut k = opcode & 0xF;
		k |= (opcode & (1 << 6)) >> 2;
		k |= (opcode & (1 << 7)) >> 2;

		let d = (((opcode >> 4 & 0xF) & 0x3) * 2 + 24) as u8;

		let rd_low = self.sram.registers[d as usize] as u16;
		let rd_high = self.sram.registers[(d + 1) as usize] as u16;
		let rd = (rd_high << 8) | rd_low;

		let result = rd + k;

		let result_low = (result & 0xFF) as u8;
		let result_high = ((result >> 8) & 0xFF) as u8;

		self.sram.registers[d as usize] = result_low;
		self.sram.registers[(d + 1) as usize] = result_high;

		let r = all_bits_u16(result);
		let rdh = all_bits_u8(result_high);

		self.status.V = !rdh.7 & r.15 == 1;
		self.status.N = r.15 == 1;
		self.status.S = (self.status.N as u8) ^ (self.status.V as u8) == 1;
		self.status.Z = result == 0;
		self.status.C = !r.15 & rdh.7 == 1;

		self.pc += 1;
		self.cycles += 2;
	}

	fn sub(&mut self, opcode: u16) {}

	fn subi(&mut self, opcode: u16) {}

	fn sbc(&mut self, opcode: u16) {}

	fn sbci(&mut self, opcode: u16) {}

	fn sbiw(&mut self, opcode: u16) {}

	fn and(&mut self, opcode: u16) {
		// 0010 00rd dddd rrrr

		let mut rd = ((opcode & 0xF0) >> 4) as u8;
		let mut rr = (opcode & 0xF) as u8;

		match high_byte(opcode) {
			0x21 => rd += 16,
			0x22 => rr += 16,
			0x23 => {
				rd += 16;
				rr += 16;
			}
			_ => {}
		}

		let result = self.sram.registers[rd as usize] & self.sram.registers[rr as usize];

		let r = all_bits_u8(result);

		self.status.V = false;
		self.status.N = r.7 == 1;
		self.status.Z = result == 0;
		self.status.S = (self.status.N as u8) ^ (self.status.V as u8) == 1;

		self.sram.registers[rd as usize] = result;

		self.pc += 1;
		self.cycles += 1;
	}

	fn andi(&mut self, opcode: u16) {}

	fn or(&mut self, opcode: u16) {}

	fn ori(&mut self, opcode: u16) {}

	fn eor(&mut self, opcode: u16) {}

	fn com(&mut self, opcode: u16) {}

	fn neg(&mut self, opcode: u16) {}

	fn sbr(&mut self, opcode: u16) {}

	fn cbr(&mut self, opcode: u16) {}

	fn inc(&mut self, opcode: u16) {}

	fn dec(&mut self, opcode: u16) {}

	fn tst(&mut self, opcode: u16) {}

	fn clr(&mut self, opcode: u16) {}

	fn ser(&mut self, opcode: u16) {}

	fn mul(&mut self, opcode: u16) {}

	fn muls(&mut self, opcode: u16) {}

	fn mulsu(&mut self, opcode: u16) {}

	fn fmul(&mut self, opcode: u16) {}

	fn fmuls(&mut self, opcode: u16) {}

	fn fmulsu(&mut self, opcode: u16) {}

	fn rjmp(&mut self, opcode: u16) {}

	fn ijmp(&mut self) {}

	fn jmp(&mut self, opcode: u16) {}

	fn rcall(&mut self, opcode: u16) {}

	fn icall(&mut self) {}

	fn call(&mut self, opcode: u16) {}

	fn ret(&mut self) {}

	fn reti(&mut self) {}

	fn cpse(&mut self, opcode: u16) {}

	fn cp(&mut self, opcode: u16) {}

	fn cpc(&mut self, opcode: u16) {}

	fn cpi(&mut self, opcode: u16) {}

	fn sbrc(&mut self, opcode: u16) {}

	fn sbrs(&mut self, opcode: u16) {}

	fn sbic(&mut self, opcode: u16) {}

	fn sbis(&mut self, opcode: u16) {}

	fn brbs(&mut self, opcode: u16) {}

	fn brbc(&mut self, opcode: u16) {}

	fn breq(&mut self, opcode: u16) {}

	fn brne(&mut self, opcode: u16) {}

	fn brcs(&mut self, opcode: u16) {}

	fn brcc(&mut self, opcode: u16) {}

	fn brsh(&mut self, opcode: u16) {}

	fn brlo(&mut self, opcode: u16) {}

	fn brmi(&mut self, opcode: u16) {}

	fn brpl(&mut self, opcode: u16) {}

	fn brge(&mut self, opcode: u16) {}

	fn brlt(&mut self, opcode: u16) {}

	fn brhs(&mut self, opcode: u16) {}

	fn brhc(&mut self, opcode: u16) {}

	fn brts(&mut self, opcode: u16) {}

	fn brtc(&mut self, opcode: u16) {}

	fn brvs(&mut self, opcode: u16) {}

	fn brvc(&mut self, opcode: u16) {}

	fn brie(&mut self, opcode: u16) {}

	fn brid(&mut self, opcode: u16) {}

	fn sbi(&mut self, opcode: u16) {}

	fn cbi(&mut self, opcode: u16) {}

	fn lsl(&mut self, opcode: u16) {}

	fn lsr(&mut self, opcode: u16) {}

	fn rol(&mut self, opcode: u16) {}

	fn ror(&mut self, opcode: u16) {}

	fn asr(&mut self, opcode: u16) {}

	fn swap(&mut self, opcode: u16) {}

	fn bset(&mut self, opcode: u16) {}

	fn bclr(&mut self, opcode: u16) {}

	fn bst(&mut self, opcode: u16) {}

	fn bld(&mut self, opcode: u16) {}

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

	fn mov(&mut self, opcode: u16) {}

	fn movw(&mut self, opcode: u16) {}

	fn ldi(&mut self, opcode: u16) {}

	fn ld(&mut self, opcode: u16) {}

	fn ldd(&mut self, opcode: u16) {}

	fn lds(&mut self, opcode: u16) {}

	fn st(&mut self, opcode: u16) {}

	fn std(&mut self, opcode: u16) {}

	fn sts(&mut self, opcode: u16) {}

	fn lpm(&mut self) {}

	fn spm(&mut self) {}

	fn in_(&mut self, opcode: u16) {}

	fn out(&mut self, opcode: u16) {}

	fn push(&mut self, opcode: u16) {}

	fn pop(&mut self, opcode: u16) {}

	pub fn nop(&mut self) {
		self.cycles += 1;
	}

	fn sleep(&mut self) {}

	fn wdr(&mut self) {}

	fn break_(&mut self) {}

	pub fn step(&mut self) {
		let opcode = self.system.program_memory.read(self.pc);

		match opcode {
			0x0000..=0x00FF => match (opcode & 0xFF) as u8 {
				0x00 => self.nop(),
				_ => panic!("Reserved opcode: {:x?}", opcode),
			},
			0x0100..=0x01FF => self.movw(opcode),
			0x0200..=0x02FF => self.muls(opcode),
			// mulsu, fmul, fmuls, fmulsu
			0x0300..=0x03FF => {}
			0x0400..=0x07FF => self.cpc(opcode),
			0x0800..=0x0BFF => self.sbc(opcode),
			0x0C00..=0x0FFF => self.add(opcode),
			0x1000..=0x13FF => self.cpse(opcode),
			0x1400..=0x17FF => self.cp(opcode),
			0x1800..=0x1BFF => self.sub(opcode),
			0x1C00..=0x1FFF => self.adc(opcode),
			0x2000..=0x23FF => self.and(opcode),
			0x2400..=0x27FF => self.eor(opcode),
			0x2800..=0x2BFF => self.or(opcode),
			0x2C00..=0x2FFF => self.mov(opcode),
			0x3000..=0x3FFF => self.cpi(opcode),
			0x4000..=0x4FFF => self.sbci(opcode),
			0x5000..=0x5FFF => self.subi(opcode),
			0x6000..=0x6FFF => self.ori(opcode),
			0x7000..=0x7FFF => self.andi(opcode),
			0x8000..=0x81FF => self.ldd(opcode),
			0x8200..=0x83FF => self.std(opcode),
			0x8400..=0x85FF => self.ldd(opcode),
			0x8600..=0x87FF => self.std(opcode),
			0x8800..=0x89FF => self.ldd(opcode),
			0x8A00..=0x8BFF => self.std(opcode),
			0x8C00..=0x8DFF => self.ldd(opcode),
			0x8E00..=0x8FFF => self.std(opcode),
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
			0x9600..=0x96FF => self.adiw(opcode),
			0x9700..=0x97FF => self.sbiw(opcode),
			0x9800..=0x98FF => self.cbi(opcode),
			0x9900..=0x99FF => self.sbic(opcode),
			0x9A00..=0x9AFF => self.sbi(opcode),
			0x9B00..=0x9BFF => self.sbis(opcode),
			0x9C00..=0x9FFF => self.mul(opcode),
			0xA000..=0xA1FF => self.ldd(opcode),
			0xA200..=0xA3FF => self.std(opcode),
			0xA400..=0xA5FF => self.ldd(opcode),
			0xA600..=0xA7FF => self.std(opcode),
			0xA800..=0xA9FF => self.ldd(opcode),
			0xAA00..=0xABFF => self.std(opcode),
			0xAC00..=0xADFF => self.ldd(opcode),
			0xAE00..=0xAFFF => self.std(opcode),
			0xB000..=0xB7FF => self.in_(opcode),
			0xB800..=0xBFFF => self.out(opcode),
			0xC000..=0xCFFF => self.rjmp(opcode),
			0xD000..=0xDFFF => self.rcall(opcode),
			0xE000..=0xEFFF => self.ldi(opcode),
			// brcs, breq, brhs, brie, brlt, brmi, brts, brvs
			0xF000..=0xF3FF => {
				//
			}
			// brcc, brge, brhc, brid, brne, brpl, brtc, brvc
			0xF400..=0xF7FF => {
				//
			}
			0xF800..=0xF9FF => self.bld(opcode),
			0xFA00..=0xFBFF => self.bst(opcode),
			0xFC00..=0xFDFF => self.sbrc(opcode),
			0xFE00..=0xFFFF => self.sbrs(opcode),
		}
	}
}
