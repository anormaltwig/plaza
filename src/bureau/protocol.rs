use crate::core::math::Vector3;

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
	SMsgUnnamed1 = 8,

	SMsgUserCount = 11,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
/// Easily read values from an incoming packet.
pub trait ByteReader {
	fn read_string(&self, start: usize) -> String;
	fn read_f32(&self, start: usize) -> f32;
	fn read_u32(&self, start: usize) -> u32;
	fn read_u16(&self, start: usize) -> u16;
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

	fn read_u16(&self, start: usize) -> u16 {
		(self[start] as u16) << 8 | (self[start + 1] as u16)
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
pub struct ByteStream {
	pub bytes: Vec<u8>,
}

#[allow(dead_code)]
impl ByteStream {
	pub fn new() -> ByteStream {
		ByteStream { bytes: Vec::new() }
	}

	pub fn general_message(id1: i32, id2: i32, opcode: Opcode, content: &ByteStream) -> ByteStream {
		let mut stream = Self::new();
		stream.write_u8(0);
		stream.write_i32(id1);
		stream.write_i32(id2);
		stream.write_u32(opcode as u32);
		stream.write_u32(content.bytes.len() as u32);
		stream.bytes.extend(&content.bytes);

		stream
	}

	pub fn position_update(id: i32, pos: &Vector3) -> ByteStream {
		let mut stream = Self::new();
		stream.write_u8(2);
		stream.write_i32(id);
		stream.write_i32(id);
		stream.write_i32(id);
		stream.write_f32(pos.x);
		stream.write_f32(pos.y);
		stream.write_f32(pos.z);

		// ???
		stream.write_u8(1);
		stream.write_u8(0);

		stream
	}

	pub fn message_common(
		id1: i32,
		id2: i32,
		id3: i32,
		msg_type: MsgCommon,
		strategy: u8,
		content: &ByteStream,
	) -> ByteStream {
		let mut common = Self::new();
		common.write_i32(id3);
		common.write_u32(msg_type as u32);
		common.write_u8(strategy);
		common.write_arr(&content.bytes);

		Self::general_message(id1, id2, Opcode::MsgCommon, &common)
	}

	pub fn write_f32(&mut self, n: f32) {
		self.write_i32((n * (0xFFFF as f32)) as i32)
	}

	pub fn write_i32(&mut self, n: i32) {
		for b in n.to_be_bytes() {
			self.bytes.push(b);
		}
	}

	pub fn write_i16(&mut self, n: i16) {
		for b in n.to_be_bytes() {
			self.bytes.push(b);
		}
	}

	pub fn write_i8(&mut self, n: i8) {
		self.bytes.push(n as u8);
	}

	pub fn write_u32(&mut self, n: u32) {
		for b in n.to_be_bytes() {
			self.bytes.push(b);
		}
	}

	pub fn write_u16(&mut self, n: u16) {
		for b in n.to_be_bytes() {
			self.bytes.push(b);
		}
	}

	pub fn write_u8(&mut self, n: u8) {
		self.bytes.push(n);
	}

	/// Write every byte of `arr` to the stream.
	pub fn write_arr(&mut self, arr: &[u8]) {
		for b in arr {
			self.bytes.push(*b);
		}
	}

	/// Writes a string to the stream that's terminated by null.
	pub fn write_string(&mut self, s: &String) {
		for b in s.as_bytes() {
			self.bytes.push(*b);
		}
		self.bytes.push(0); // Append null char.
	}
}
