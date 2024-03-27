use crate::math::Vector3;

#[allow(dead_code)]
#[repr(u32)]
pub enum Opcode {
	CMsgNewUser = 0,
	SMsgClientId = 1,
	SMsgUserJoined = 2,
	SMsgUserLeft = 3,
	SMsgBroadcastId = 4,

	MsgCommon = 6,
	CMsgStateChange = 7,
	SMsgSetMaster = 8,

	SMsgUserCount = 11,
}

#[repr(u32)]
pub enum MsgCommon {
	TransformUpdate = 2,
	ChatSend = 9,
	CharacterUpdate = 12,
	NameChange = 13,
	AvatarChange = 14,
	PrivateChat = 15,
	ApplSpecific = 10000,
}

/// Easily read values from an incoming packet.
pub trait ByteReader {
	fn read_string(&self, start: usize) -> String;
	fn read_f32(&self, start: usize) -> f32;
	fn read_u32(&self, start: usize) -> u32;
	fn read_u8(&self, start: usize) -> u8;
	fn read_i32(&self, start: usize) -> i32;
	fn read_i8(&self, start: usize) -> i8;
}

impl ByteReader for [u8] {
	fn read_string(&self, start: usize) -> String {
		let mut buf = vec![0; self.len().checked_sub(start).unwrap_or(0)];
		let mut i = 0;
		for b in &self[start..] {
			let b = *b;
			if b == 0 {
				break;
			}
			buf[i] = b;
			i += 1;
		}

		String::from_utf8(buf[..i].to_vec()).unwrap_or(String::new())
	}

	fn read_f32(&self, start: usize) -> f32 {
		let n = i32::from_be_bytes([
			self[start],
			self[start + 1],
			self[start + 2],
			self[start + 3],
		]);

		(n as f32) / 65535.0
	}

	fn read_u32(&self, start: usize) -> u32 {
		u32::from_be_bytes([
			self[start],
			self[start + 1],
			self[start + 2],
			self[start + 3],
		])
	}

	fn read_u8(&self, start: usize) -> u8 {
		self[start]
	}

	fn read_i32(&self, start: usize) -> i32 {
		i32::from_be_bytes([
			self[start],
			self[start + 1],
			self[start + 2],
			self[start + 3],
		])
	}

	fn read_i8(&self, start: usize) -> i8 {
		self[start] as i8
	}
}

/// Easily write values to byte vector for networking.
/// All functions are Big Endian.
pub struct ByteWriter {
	pub bytes: Vec<u8>,
}

#[allow(dead_code)]
impl ByteWriter {
	pub fn new() -> ByteWriter {
		ByteWriter { bytes: Vec::new() }
	}

	pub fn general_message(id1: i32, id2: i32, opcode: Opcode, content: &ByteWriter) -> ByteWriter {
		let mut stream = Self::new()
			.write_u8(0)
			.write_i32(id1)
			.write_i32(id2)
			.write_u32(opcode as u32)
			.write_u32(content.bytes.len() as u32);
		stream.bytes.extend(&content.bytes);

		stream
	}

	pub fn position_update(id: i32, pos: &Vector3) -> ByteWriter {
		let stream = Self::new()
			.write_u8(2)
			.write_i32(id)
			.write_i32(id)
			.write_i32(id)
			.write_f32(pos.x)
			.write_f32(pos.y)
			.write_f32(pos.z)
			// ???
			.write_u8(1)
			.write_u8(0);

		stream
	}

	pub fn message_common(
		id1: i32,
		id2: i32,
		id3: i32,
		msg_type: MsgCommon,
		strategy: u8,
		content: &ByteWriter,
	) -> ByteWriter {
		let common = Self::new()
			.write_i32(id3)
			.write_u32(msg_type as u32)
			.write_u8(strategy)
			.write_arr(&content.bytes);

		Self::general_message(id1, id2, Opcode::MsgCommon, &common)
	}

	pub fn write_f32(self, n: f32) -> Self {
		self.write_i32((n * (0xFFFF as f32)) as i32)
	}

	pub fn write_i32(mut self, n: i32) -> Self {
		for b in n.to_be_bytes() {
			self.bytes.push(b);
		}

		self
	}

	pub fn write_i16(mut self, n: i16) -> Self {
		for b in n.to_be_bytes() {
			self.bytes.push(b);
		}

		self
	}

	pub fn write_i8(mut self, n: i8) -> Self {
		self.bytes.push(n as u8);

		self
	}

	pub fn write_u32(mut self, n: u32) -> Self {
		for b in n.to_be_bytes() {
			self.bytes.push(b);
		}

		self
	}

	pub fn write_u16(mut self, n: u16) -> Self {
		for b in n.to_be_bytes() {
			self.bytes.push(b);
		}

		self
	}

	pub fn write_u8(mut self, n: u8) -> Self {
		self.bytes.push(n);

		self
	}

	/// Write every byte of `arr` to the stream.
	pub fn write_arr(mut self, arr: &[u8]) -> Self {
		for b in arr {
			self.bytes.push(*b);
		}

		self
	}

	/// Writes a string to the stream that's terminated by null.
	pub fn write_string(mut self, s: &str) -> Self {
		for b in s.as_bytes() {
			self.bytes.push(*b);
		}
		self.bytes.push(0); // Append null char.

		self
	}
}
