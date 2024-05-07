use super::math::Vector3;

// Documentation of types listed here should be found in VSCP.md.

#[repr(u32)]
pub enum Opcode {
	// CMsgNewUser = 0,
	SMsgClientId = 1,
	SMsgUserJoined = 2,
	SMsgUserLeft = 3,
	SMsgBroadcastId = 4,

	MsgCommon = 6,
	// CMsgStateChange = 7,
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

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Strategy {
	AuraClients = 0,
	AuraClientsExceptSender = 1,
	SpecificClient = 2,
	AllClients = 3,
	AllClientsExceptSender = 4,
	Unknown5 = 5,
	Unknown6 = 6,

	Invalid = u8::MAX,
}

impl From<u8> for Strategy {
	fn from(value: u8) -> Self {
		match value {
			0 => Strategy::AuraClients,
			1 => Strategy::AuraClientsExceptSender,
			2 => Strategy::SpecificClient,
			3 => Strategy::AllClients,
			4 => Strategy::AllClientsExceptSender,
			5 => Strategy::Unknown5,
			6 => Strategy::Unknown6,
			_ => Strategy::Invalid,
		}
	}
}

pub trait ByteReader {
	fn read_string(&self, start: usize) -> String;
	fn read_f32(&self, start: usize) -> f32;
	fn read_u32(&self, start: usize) -> u32;
	fn read_i32(&self, start: usize) -> i32;
}

impl ByteReader for [u8] {
	fn read_string(&self, start: usize) -> String {
		let mut buf = vec![0; self.len().saturating_sub(start)];
		let mut i = 0;
		for b in &self[start..] {
			let b = *b;
			if b == 0 {
				break;
			}
			buf[i] = b;
			i += 1;
		}

		String::from_utf8(buf[..i].to_vec()).unwrap_or_default()
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

	fn read_i32(&self, start: usize) -> i32 {
		i32::from_be_bytes([
			self[start],
			self[start + 1],
			self[start + 2],
			self[start + 3],
		])
	}
}

/// Easily write values to byte vector for networking.
/// All functions are Big Endian.
pub struct ByteWriter {
	pub bytes: Vec<u8>,
}

impl ByteWriter {
	pub fn new(n: usize) -> Self {
		Self {
			bytes: Vec::with_capacity(n),
		}
	}

	pub fn general_message(id1: i32, id2: i32, opcode: Opcode, content: &[u8]) -> Self {
		Self::new(17 + content.len())
			.write_u8(0)
			.write_i32(id1)
			.write_i32(id2)
			.write_u32(opcode as u32)
			.write_u32(content.len() as u32)
			.write_arr(content)
	}

	pub fn position_update(id: i32, pos: &Vector3) -> Self {
		Self::new(27)
			.write_u8(2)
			.write_i32(id)
			.write_i32(id)
			.write_i32(id)
			.write_f32(pos.x)
			.write_f32(pos.y)
			.write_f32(pos.z)
			// ???
			.write_u8(1)
			.write_u8(0)
	}

	pub fn message_common(
		id1: i32,
		id2: i32,
		msg_type: MsgCommon,
		strategy: Strategy,
		content: &[u8],
	) -> Self {
		Self::general_message(
			id1,
			id1,
			Opcode::MsgCommon,
			&Self::new(9 + content.len())
				.write_i32(id2)
				.write_u32(msg_type as u32)
				.write_u8(strategy as u8)
				.write_arr(content)
				.bytes,
		)
	}

	pub fn write_f32(self, n: f32) -> Self {
		self.write_i32((n * (0xFFFF as f32)) as i32)
	}

	pub fn write_i32(mut self, n: i32) -> Self {
		self.bytes.extend(n.to_be_bytes());

		self
	}

	pub fn write_u32(mut self, n: u32) -> Self {
		self.bytes.extend(n.to_be_bytes());

		self
	}

	pub fn write_u8(mut self, n: u8) -> Self {
		self.bytes.push(n);

		self
	}

	/// Write every byte of `arr` to the stream.
	pub fn write_arr(mut self, arr: &[u8]) -> Self {
		self.bytes.extend(arr);

		self
	}

	/// Writes a string to the stream that's terminated by null.
	pub fn write_string(mut self, s: &str) -> Self {
		self.bytes.extend(s.as_bytes());
		self.bytes.push(0); // Append null char.

		self
	}
}
