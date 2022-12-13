use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::ops::Range;

const PROGRAM_FLASH_RANGE: Range<u16> = 0x0000..0x4000;
const APP_FLASH_RANGE: Range<u16> = 0x0000..0x3800;
const BOOT_FLASH_RANGE: Range<u16> = 0x3800..0x4000;
const SRAM_RANGE: Range<u16> = 0x0000..0x0900;
const EEPROM_RANGE: Range<u16> = 0x0000..0x0400;

const APP_FLASH_SIZE: u16 = 0x7800;
const BOOT_FLASH_SIZE: u16 = 0x800;
const EEPROM_SIZE: u16 = 0x400;

//------------------ Programmable Flash Memory --------------------------------

pub trait Memory {
	fn address_range(&self) -> &Range<u16>;
	fn read(&mut self, address: u16) -> u16;
	fn write(&mut self, address: u16, data: u16);
}

#[derive(Debug)]
pub struct ApplicationFlash {
	pub data: Vec<u16>,
}

impl Memory for ApplicationFlash {
	fn address_range(&self) -> &Range<u16> {
		&APP_FLASH_RANGE
	}

	fn read(&mut self, address: u16) -> u16 {
		self.data[(address as usize)]
	}

	fn write(&mut self, address: u16, data: u16) {
		self.data[(address as usize)] = data;
	}
}

impl Default for ApplicationFlash {
	fn default() -> Self {
		Self {
			data: vec![0; APP_FLASH_SIZE as usize],
		}
	}
}

#[derive(Debug)]
pub struct BootFlash {
	pub data: Vec<u16>,
}

impl Memory for BootFlash {
	fn address_range(&self) -> &Range<u16> {
		&BOOT_FLASH_RANGE
	}

	fn read(&mut self, address: u16) -> u16 {
		let mapped_address = address - 0x3800;
		self.data[(mapped_address as usize)]
	}

	fn write(&mut self, address: u16, data: u16) {
		let mapped_address = address - 0x3800;
		self.data[(mapped_address as usize)] = data;
	}
}

impl Default for BootFlash {
	fn default() -> Self {
		Self {
			data: vec![0; BOOT_FLASH_SIZE as usize],
		}
	}
}

#[derive(Default, Debug)]
pub struct ProgramMemory {
	pub app_flash: ApplicationFlash,
	pub boot_flash: BootFlash,
}

impl Memory for ProgramMemory {
	fn address_range(&self) -> &Range<u16> {
		&PROGRAM_FLASH_RANGE
	}

	fn read(&mut self, address: u16) -> u16 {
		if self.app_flash.address_range().contains(&address) {
			self.app_flash.read(address)
		} else if self.boot_flash.address_range().contains(&address) {
			self.boot_flash.read(address)
		} else {
			panic!("Program memory does not contain address 0x{:x?}", address);
		}
	}

	fn write(&mut self, address: u16, data: u16) {
		if self.app_flash.address_range().contains(&address) {
			self.app_flash.write(address, data);
		} else if self.boot_flash.address_range().contains(&address) {
			self.boot_flash.write(address, data);
		} else {
			panic!("Program memory does not contain address 0x{:x?}", address);
		}
	}
}

//------------------ EEPROM Memory --------------------------------------------

pub struct EepromMemory {
	data: Vec<u8>,
}

impl Default for EepromMemory {
	fn default() -> Self {
		Self {
			data: vec![0; EEPROM_SIZE as usize],
		}
	}
}

impl Memory for EepromMemory {
	fn address_range(&self) -> &Range<u16> {
		&EEPROM_RANGE
	}

	fn read(&mut self, address: u16) -> u16 {
		if !self.address_range().contains(&address) {
			panic!("EEPROM memory does not contain address 0x{:x?}", address);
		} else {
			self.data[address as usize] as u16
		}
	}

	fn write(&mut self, address: u16, data: u16) {
		if !self.address_range().contains(&address) {
			panic!("EEPROM memory does not contain address 0x{:x?}", address);
		} else {
			self.data[address as usize] = data as u8
		}
	}
}

//------------------ SRAM -----------------------------------------------------

#[derive(Debug)]
pub struct Sram {
	pub registers: Vec<u8>,
	pub io_registers: Vec<u8>,
	pub ext_io_registers: Vec<u8>,
	pub internal_data: Vec<u8>,
}

impl Default for Sram {
	fn default() -> Self {
		Self {
			registers: vec![0; 32],
			io_registers: vec![0; 64],
			ext_io_registers: vec![0; 160],
			internal_data: vec![0; 2048],
		}
	}
}

impl Memory for Sram {
	fn address_range(&self) -> &Range<u16> {
		&SRAM_RANGE
	}

	fn read(&mut self, address: u16) -> u16 {
		match address {
			0x0000..=0x001F => self.registers[address as usize] as u16,
			0x0020..=0x005F => {
				let mapped_address = address - 0x0020;
				self.io_registers[mapped_address as usize] as u16
			}
			0x0060..=0x00FF => {
				let mapped_address = address - 0x0060;
				self.ext_io_registers[mapped_address as usize] as u16
			}
			0x0100..=0x08FF => {
				let mapped_address = address - 0x0100;
				self.internal_data[mapped_address as usize] as u16
			}
			_ => panic!("SRAM does not contain address 0x{:x?}", address),
		}
	}

	fn write(&mut self, _address: u16, _data: u16) {
		todo!();
	}
}

lazy_static! {
	pub static ref REGISTER_NAMES: BTreeMap<u8, String> = {
		let mut sram: BTreeMap<u8, String> = BTreeMap::new();
		sram.insert(0x0, "R0".to_string());
		sram.insert(0x1, "R1".to_string());
		sram.insert(0x2, "R2".to_string());
		sram.insert(0x3, "R3".to_string());
		sram.insert(0x4, "R4".to_string());
		sram.insert(0x5, "R5".to_string());
		sram.insert(0x6, "R6".to_string());
		sram.insert(0x7, "R7".to_string());
		sram.insert(0x8, "R8".to_string());
		sram.insert(0x9, "R9".to_string());
		sram.insert(0xA, "R10".to_string());
		sram.insert(0xB, "R11".to_string());
		sram.insert(0xC, "R12".to_string());
		sram.insert(0xD, "R13".to_string());
		sram.insert(0xE, "R14".to_string());
		sram.insert(0xF, "R15".to_string());
		sram.insert(0x10, "R16".to_string());
		sram.insert(0x11, "R17".to_string());
		sram.insert(0x12, "R18".to_string());
		sram.insert(0x13, "R19".to_string());
		sram.insert(0x14, "R20".to_string());
		sram.insert(0x15, "R21".to_string());
		sram.insert(0x16, "R22".to_string());
		sram.insert(0x17, "R23".to_string());
		sram.insert(0x18, "R24".to_string());
		sram.insert(0x19, "R25".to_string());
		sram.insert(0x1A, "XL".to_string());
		sram.insert(0x1B, "XH".to_string());
		sram.insert(0x1C, "YL".to_string());
		sram.insert(0x1D, "YH".to_string());
		sram.insert(0x1E, "ZL".to_string());
		sram.insert(0x1F, "ZH".to_string());
		sram.insert(0x20, "Reserved".to_string());
		sram.insert(0x21, "Reserved".to_string());
		sram.insert(0x22, "Reserved".to_string());
		sram.insert(0x23, "PINB".to_string());
		sram.insert(0x24, "DDRB".to_string());
		sram.insert(0x25, "PORTB".to_string());
		sram.insert(0x26, "PINC".to_string());
		sram.insert(0x27, "DDRC".to_string());
		sram.insert(0x28, "PORTC".to_string());
		sram.insert(0x29, "PIND".to_string());
		sram.insert(0x2A, "DDRD".to_string());
		sram.insert(0x2B, "PORTD".to_string());
		sram.insert(0x2C, "Reserved".to_string());
		sram.insert(0x2D, "Reserved".to_string());
		sram.insert(0x2E, "Reserved".to_string());
		sram.insert(0x2F, "Reserved".to_string());
		sram.insert(0x30, "Reserved".to_string());
		sram.insert(0x31, "Reserved".to_string());
		sram.insert(0x32, "Reserved".to_string());
		sram.insert(0x33, "Reserved".to_string());
		sram.insert(0x34, "Reserved".to_string());
		sram.insert(0x35, "TIFR0".to_string());
		sram.insert(0x36, "TIFR1".to_string());
		sram.insert(0x37, "TIFR2".to_string());
		sram.insert(0x38, "Reserved".to_string());
		sram.insert(0x39, "Reserved".to_string());
		sram.insert(0x3A, "Reserved".to_string());
		sram.insert(0x3B, "PCIFR".to_string());
		sram.insert(0x3C, "EIFR".to_string());
		sram.insert(0x3D, "EIMSK".to_string());
		sram.insert(0x3E, "GPIOR0".to_string());
		sram.insert(0x3F, "EECR".to_string());
		sram.insert(0x40, "EEDR".to_string());
		sram.insert(0x41, "EEARL".to_string());
		sram.insert(0x42, "EEARH".to_string());
		sram.insert(0x43, "GTCCR".to_string());
		sram.insert(0x44, "TCCR0A".to_string());
		sram.insert(0x45, "TCCR0B".to_string());
		sram.insert(0x46, "TCNT0".to_string());
		sram.insert(0x47, "OCR0A".to_string());
		sram.insert(0x48, "OCR0B".to_string());
		sram.insert(0x49, "Reserved".to_string());
		sram.insert(0x4A, "GPIOR1".to_string());
		sram.insert(0x4B, "GPIOR2".to_string());
		sram.insert(0x4C, "SPCR".to_string());
		sram.insert(0x4D, "SPSR".to_string());
		sram.insert(0x4E, "SPDR".to_string());
		sram.insert(0x4F, "Reserved".to_string());
		sram.insert(0x50, "ACSR".to_string());
		sram.insert(0x51, "Reserved".to_string());
		sram.insert(0x52, "Reserved".to_string());
		sram.insert(0x53, "SMCR".to_string());
		sram.insert(0x54, "MCUSR".to_string());
		sram.insert(0x55, "MCUCR".to_string());
		sram.insert(0x56, "Reserved".to_string());
		sram.insert(0x57, "SPMCSR".to_string());
		sram.insert(0x58, "Reserved".to_string());
		sram.insert(0x59, "Reserved".to_string());
		sram.insert(0x5A, "Reserved".to_string());
		sram.insert(0x5B, "Reserved".to_string());
		sram.insert(0x5C, "Reserved".to_string());
		sram.insert(0x5D, "SPL".to_string());
		sram.insert(0x5E, "SPH".to_string());
		sram.insert(0x5F, "SREG".to_string());
		sram.insert(0x60, "WDTCSR".to_string());
		sram.insert(0x61, "CLKPR".to_string());
		sram.insert(0x62, "Reserved".to_string());
		sram.insert(0x63, "Reserved".to_string());
		sram.insert(0x64, "PRR".to_string());
		sram.insert(0x65, "Reserved".to_string());
		sram.insert(0x66, "OSCCAL".to_string());
		sram.insert(0x67, "Reserved".to_string());
		sram.insert(0x68, "PCICR".to_string());
		sram.insert(0x69, "EICRA".to_string());
		sram.insert(0x6A, "Reserved".to_string());
		sram.insert(0x6B, "PCMSK0".to_string());
		sram.insert(0x6C, "PCMSK1".to_string());
		sram.insert(0x6D, "PCMSK2".to_string());
		sram.insert(0x6E, "TIMSK0".to_string());
		sram.insert(0x6F, "TIMSK1".to_string());
		sram.insert(0x70, "TIMSK2".to_string());
		sram.insert(0x71, "Reserved".to_string());
		sram.insert(0x72, "Reserved".to_string());
		sram.insert(0x73, "Reserved".to_string());
		sram.insert(0x74, "Reserved".to_string());
		sram.insert(0x75, "Reserved".to_string());
		sram.insert(0x76, "Reserved".to_string());
		sram.insert(0x77, "Reserved".to_string());
		sram.insert(0x78, "ADCL".to_string());
		sram.insert(0x79, "ADCH".to_string());
		sram.insert(0x7A, "ADCSRA".to_string());
		sram.insert(0x7B, "ADCSRB".to_string());
		sram.insert(0x7C, "ADMUX".to_string());
		sram.insert(0x7D, "Reserved".to_string());
		sram.insert(0x7E, "DIDR0".to_string());
		sram.insert(0x7F, "DIDR1".to_string());
		sram.insert(0x80, "TCCR1A".to_string());
		sram.insert(0x81, "TCCR1B".to_string());
		sram.insert(0x82, "TCCR1C".to_string());
		sram.insert(0x83, "Reserved".to_string());
		sram.insert(0x84, "TCNT1L".to_string());
		sram.insert(0x85, "TCNT1H".to_string());
		sram.insert(0x86, "ICR1L".to_string());
		sram.insert(0x87, "ICR1H".to_string());
		sram.insert(0x88, "OCR1AL".to_string());
		sram.insert(0x89, "OCR1AH".to_string());
		sram.insert(0x8A, "OCR1BL".to_string());
		sram.insert(0x8B, "OCR1BH".to_string());
		for x in 0x8C..=0xAF {
			sram.insert(x, "Reserved".to_string());
		}
		sram.insert(0xB0, "TCCR2A".to_string());
		sram.insert(0xB1, "TCCR2B".to_string());
		sram.insert(0xB2, "TCNT2".to_string());
		sram.insert(0xB3, "OCR2A".to_string());
		sram.insert(0xB4, "OCR2B".to_string());
		sram.insert(0xB5, "Reserved".to_string());
		sram.insert(0xB6, "ASSR".to_string());
		sram.insert(0xB7, "Reserved".to_string());
		sram.insert(0xB8, "TWBR".to_string());
		sram.insert(0xB9, "TWSR".to_string());
		sram.insert(0xBA, "TWAR".to_string());
		sram.insert(0xBB, "TWDR".to_string());
		sram.insert(0xBC, "TWCR".to_string());
		sram.insert(0xBD, "TWAMR".to_string());
		sram.insert(0xBE, "Reserved".to_string());
		sram.insert(0xBF, "Reserved".to_string());
		sram.insert(0xC0, "UCSR0A".to_string());
		sram.insert(0xC1, "UCSR0B".to_string());
		sram.insert(0xC2, "UCSR0C".to_string());
		sram.insert(0xC3, "Reserved".to_string());
		sram.insert(0xC4, "UBRR0L".to_string());
		sram.insert(0xC5, "UBRR0H".to_string());
		sram.insert(0xC6, "UDR0".to_string());
		for x in 0xC7..=0xFF {
			sram.insert(x, "Reserved".to_string());
		}
		sram
	};
}
