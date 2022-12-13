#[cfg(test)]
mod instructions {
	use crate::cpu::Cpu;
	#[test]
	fn add() {
		let mut cpu = Cpu::init();
		let prg: [u16; 4] = [0x0C00, 0x0D7A, 0x0E75, 0x0F35];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
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
		let mut cpu = Cpu::init();
		let prg: [u16; 4] = [0x1C28, 0x1D48, 0x1E5A, 0x1FCF];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.status.C = true;

		cpu.sram.registers[2] = 2;
		cpu.sram.registers[8] = 2;

		cpu.step();

		cpu.status.C = true;

		cpu.sram.registers[20] = 9;
		cpu.sram.registers[5] = 0;
		cpu.sram.registers[26] = 15;

		cpu.step();
		cpu.status.C = true;
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
		let mut cpu = Cpu::init();
		let prg: [u16; 4] = [0x9600, 0x9628, 0x96A3, 0x96FF];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
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

	#[test]
	fn sub() {
		let mut cpu = Cpu::init();
		let prg: [u16; 4] = [0x1847, 0x198C, 0x1AB3, 0x1B02];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.sram.registers[4] = 10;
		cpu.sram.registers[7] = 3;
		cpu.sram.registers[24] = 1;
		cpu.sram.registers[12] = 1;
		cpu.sram.registers[11] = 5;
		cpu.sram.registers[19] = 3;
		cpu.sram.registers[16] = 1;
		cpu.sram.registers[18] = 0;

		cpu.step();
		cpu.step();
		cpu.step();
		cpu.step();

		assert_eq!(cpu.sram.registers[4], 7);
		assert_eq!(cpu.sram.registers[24], 0);
		assert_eq!(cpu.sram.registers[11], 2);
		assert_eq!(cpu.sram.registers[16], 1);
	}

	#[test]
	fn subi() {
		let mut cpu = Cpu::init();
		let prg: [u16; 2] = [0x5135, 0x53CA];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.sram.registers[19] = 35;
		cpu.sram.registers[28] = 70;

		cpu.step();
		cpu.step();

		assert_eq!(cpu.sram.registers[19], 14);
		assert_eq!(cpu.sram.registers[28], 12);
	}

	#[test]
	fn sbc() {
		let mut cpu = Cpu::init();
		let prg: [u16; 2] = [0x08B9, 0x0B0D];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.sram.registers[9] = 10;
		cpu.sram.registers[11] = 25;
		cpu.sram.registers[16] = 6;
		cpu.sram.registers[29] = 3;

		cpu.step();
		cpu.status.C = true;
		cpu.step();

		assert_eq!(cpu.sram.registers[11], 15);
		assert_eq!(cpu.sram.registers[16], 2);
	}

	#[test]
	fn sbci() {
		let mut cpu = Cpu::init();
		let prg: [u16; 2] = [0x4048, 0x4242];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.sram.registers[20] = 88;

		cpu.step();
		cpu.status.C = true;
		cpu.step();

		assert_eq!(cpu.sram.registers[20], 45);
	}

	#[test]
	fn sbiw() {
		let mut cpu = Cpu::init();
		let prg: [u16; 1] = [0x9760];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.sram.registers[28] = 0xAA;
		cpu.sram.registers[29] = 0x55;

		cpu.step();

		assert_eq!(cpu.sram.registers[28], 0x9A);
		assert_eq!(cpu.sram.registers[29], 0x55);
	}

	#[test]
	fn and() {
		let mut cpu = Cpu::init();
		let prg: [u16; 2] = [0x2000, 0x2038];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.sram.registers[0] = 0;
		cpu.sram.registers[3] = 67;
		cpu.sram.registers[8] = 13;

		cpu.step();
		cpu.step();

		assert_eq!(cpu.sram.registers[0], 0);
		assert_eq!(cpu.sram.registers[3], 1);
	}

	#[test]
	fn andi() {
		let mut cpu = Cpu::init();
		let prg: [u16; 1] = [0x7227];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.sram.registers[18] = 30;
		cpu.step();

		assert_eq!(cpu.sram.registers[18], 6);
	}

	#[test]
	fn or() {
		let mut cpu = Cpu::init();
		let prg: [u16; 2] = [0x2800, 0x2935];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.sram.registers[0] = 72;
		cpu.sram.registers[5] = 5;
		cpu.sram.registers[19] = 3;

		cpu.step();
		cpu.step();

		assert_eq!(cpu.sram.registers[0], 72);
		assert_eq!(cpu.sram.registers[19], 7);
	}

	#[test]
	fn ori() {
		let mut cpu = Cpu::init();
		let prg: [u16; 1] = [0x614C];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.sram.registers[20] = 84;
		cpu.step();

		assert_eq!(cpu.sram.registers[20], 92);
	}

	#[test]
	fn eor() {
		let mut cpu = Cpu::init();
		let prg: [u16; 2] = [0x2700, 0x256A];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.sram.registers[16] = 39;
		cpu.sram.registers[10] = 17;
		cpu.sram.registers[22] = 98;

		cpu.step();
		cpu.step();

		assert_eq!(cpu.sram.registers[16], 0);
		assert_eq!(cpu.sram.registers[22], 115);
	}

	#[test]
	fn com() {
		let mut cpu = Cpu::init();
		let prg: [u16; 2] = [0x94B0, 0x9540];

		for (index, word) in prg.into_iter().enumerate() {
			cpu.system.program_memory.app_flash.data[0x0000 + index] = word;
		}

		cpu.sram.registers[11] = 14;
		cpu.sram.registers[20] = 99;

		cpu.step();
		cpu.step();

		assert_eq!(cpu.sram.registers[11], 241);
		assert_eq!(cpu.sram.registers[20], 156);
	}
}
