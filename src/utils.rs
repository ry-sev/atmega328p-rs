type EightBits = (u8, u8, u8, u8, u8, u8, u8, u8);

type SixteenBits = (
	u8,
	u8,
	u8,
	u8,
	u8,
	u8,
	u8,
	u8,
	u8,
	u8,
	u8,
	u8,
	u8,
	u8,
	u8,
	u8,
);

pub fn to_u16(high_byte: u8, low_byte: u8) -> u16 {
	(high_byte as u16) << 8 | (low_byte as u16)
}

pub fn low_byte(value: u16) -> u16 {
	value & 0xFF
}

pub fn high_byte(value: u16) -> u16 {
	(value >> 8) & 0xFF
}

pub fn low_nibble(value: u8) -> u8 {
	value & 0xF
}

pub fn high_nibble(value: u8) -> u8 {
	(value >> 4 & 0xF) as u8
}

pub fn bit(value: u8, bit: u8) -> u8 {
	value & (1 << bit)
}

pub fn bits_u8(value: u8) -> EightBits {
	(
		value & (1 << 0),
		value & (1 << 1),
		value & (1 << 2),
		value & (1 << 3),
		value & (1 << 4),
		value & (1 << 5),
		value & (1 << 6),
		value & (1 << 7),
	)
}

pub fn bits_u16(value: u16) -> SixteenBits {
	(
		(value & (1 << 0)) as u8,
		(value & (1 << 1)) as u8,
		(value & (1 << 2)) as u8,
		(value & (1 << 3)) as u8,
		(value & (1 << 4)) as u8,
		(value & (1 << 5)) as u8,
		(value & (1 << 6)) as u8,
		(value & (1 << 7)) as u8,
		(value & (1 << 8)) as u8,
		(value & (1 << 9)) as u8,
		(value & (1 << 10)) as u8,
		(value & (1 << 11)) as u8,
		(value & (1 << 12)) as u8,
		(value & (1 << 13)) as u8,
		(value & (1 << 14)) as u8,
		(value & (1 << 15)) as u8,
	)
}
