#[cfg(test)]
mod instructions {
	use crate::cpu::Cpu;
	use crate::system::System;
	#[test]
	fn add() {
		let mut cpu = Cpu::init(System::default());

		let prg_start = 0x0000;

		let prg: [u16; 4] = [0x0C00, 0x0D7A, 0x0E75, 0x0F35];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[prg_start + index] = word;
		}

		cpu.sram.registers[0] = 10;
		cpu.sram.registers[23] = 2;
		cpu.sram.registers[10] = 3;
		cpu.sram.registers[7] = 6;
		cpu.sram.registers[21] = 14;
		cpu.sram.registers[19] = 7;

		cpu.step();
		cpu.step();
		cpu.step();
		cpu.step();

		assert_eq!(cpu.sram.registers[0], 20);
		assert_eq!(cpu.sram.registers[23], 5);
		assert_eq!(cpu.sram.registers[7], 20);
		assert_eq!(cpu.sram.registers[19], 21);
		assert_eq!(cpu.cycles, 4);
	}

	#[test]
	fn adc() {
		let mut cpu = Cpu::init(System::default());
		let prg_start = 0x0000;
		let prg: [u16; 4] = [0x1C28, 0x1D48, 0x1E5A, 0x1FCF];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[prg_start + index] = word;
		}

		cpu.status.set_byte(0x1);

		cpu.sram.registers[2] = 2;
		cpu.sram.registers[8] = 2;

		cpu.step();

		cpu.status.set_byte(0x1);

		cpu.sram.registers[20] = 9;
		cpu.sram.registers[5] = 0;
		cpu.sram.registers[26] = 15;

		cpu.step();
		cpu.status.set_byte(0x1);
		cpu.step();

		cpu.sram.registers[28] = 5;
		cpu.sram.registers[31] = 7;

		cpu.step();

		assert_eq!(cpu.sram.registers[2], 5);
		assert_eq!(cpu.sram.registers[20], 12);
		assert_eq!(cpu.sram.registers[5], 16);
		assert_eq!(cpu.sram.registers[28], 12);
	}

	#[test]
	fn adiw() {
		let mut cpu = Cpu::init(System::default());
		let prg_start = 0x0000;
		let prg: [u16; 4] = [0x9600, 0x9628, 0x96A3, 0x96FF];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[prg_start + index] = word;
		}

		cpu.sram.registers[24] = 2;
		cpu.sram.registers[25] = 3;

		cpu.sram.registers[28] = 10;
		cpu.sram.registers[29] = 5;

		cpu.step();
		cpu.step();

		assert_eq!(cpu.sram.registers[24], 2);
		assert_eq!(cpu.sram.registers[25], 3);
		assert_eq!(cpu.sram.registers[28], 18);
		assert_eq!(cpu.sram.registers[29], 5);

		assert_eq!(cpu.status.Z, false);

		cpu.sram.registers[29] = 0;

		cpu.step();

		assert_eq!(cpu.sram.registers[28], 53);
		assert_eq!(cpu.sram.registers[29], 0);

		cpu.sram.registers[30] = 1;
		cpu.sram.registers[31] = 86;

		cpu.step();

		assert_eq!(cpu.sram.registers[30], 64);
		assert_eq!(cpu.sram.registers[31], 86);

		assert_eq!(cpu.cycles, 8);
	}
}
