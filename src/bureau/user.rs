use std::{
	collections::HashSet,
	io::{self, ErrorKind, Read, Write},
	net::{SocketAddr, TcpStream},
};

use super::{
	math::{Mat3, Vector3},
	protocol::{ByteReader, ByteWriter, MsgCommon, Opcode, Strategy},
};

pub enum UserEvent {
	NewUser(String, String),
	StateChange,
	PositionUpdate(Vector3),
	TransformUpdate(Mat3, Vector3),
	ChatSend(String),
	CharacterUpdate(String),
	NameChange(String),
	AvatarChange(String),
	PrivateChat(i32, String),
	ApplSpecific(Strategy, i32, String, String, i32),
}

fn validate_avatar(avatar: String) -> String {
	if !avatar.starts_with("avtwrl/") {
		return String::from("avtwrl/01cat.wrl");
	}

	avatar
}

pub struct User {
	pub id: i32,
	pub aura: HashSet<i32>,
	pub connected: bool,
	pub username: String,
	pub avatar: String,
	pub data: String,

	addr: SocketAddr,
	socket: TcpStream,
	position: Vector3,
	rotation: Mat3,
}

impl User {
	pub fn new(id: i32, socket: TcpStream) -> io::Result<Self> {
		Ok(Self {
			id,
			aura: HashSet::new(),
			connected: true,
			username: String::new(),
			avatar: String::new(),
			data: String::new(),

			addr: socket.peer_addr()?,
			socket,
			position: Vector3::new(0.0, 0.0, 0.0),
			rotation: Mat3::new(),
		})
	}

	/// Get user SocketAddr.
	pub fn addr(&self) -> &SocketAddr {
		&self.addr
	}

	/// Set user position.
	pub fn set_pos(&mut self, pos: Vector3) {
		self.send(&ByteWriter::position_update(self.id, &pos));
		self.position = pos;
	}
	/// Get user position.
	pub fn pos(&self) -> &Vector3 {
		&self.position
	}

	/// Set user rotation.
	pub fn set_rot(&mut self, rot: Mat3) {
		let mut transform_update = ByteWriter::new(48);

		for f in rot.data {
			transform_update = transform_update.write_f32(f);
		}

		transform_update = transform_update
			.write_f32(self.position.x)
			.write_f32(self.position.y)
			.write_f32(self.position.z);

		self.send(&ByteWriter::message_common(
			self.id,
			self.id,
			MsgCommon::TransformUpdate,
			Strategy::AuraClients,
			&transform_update.bytes,
		));
		self.rotation = rot;
	}

	/// Send all data within a ByteWriter to the socket this User contains.
	pub fn send(&mut self, stream: &ByteWriter) {
		if self.socket.write_all(&stream.bytes).is_err() {
			self.connected = false;
		}
	}

	fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
		match self.socket.read(buf) {
			Ok(count) => {
				if count == 0 {
					self.connected = false;
					return None;
				}

				Some(count)
			}
			Err(e) if e.kind() == ErrorKind::WouldBlock => None,
			Err(_) => {
				self.connected = false;
				None
			}
		}
	}

	/// Poll a single event from this User.
	pub fn poll(&mut self) -> Option<UserEvent> {
		let mut buf: [u8; 1] = [0];
		let n = self.read(&mut buf)?;
		if n == 0 {
			return None;
		}

		let packet_type = buf[0];

		match packet_type {
			0 => self.general_message(),
			1 => {
				// I don't know what this type does. I do know its most likely 14 bytes.
				// To avoid outright disconnecting the user for sending a packet that should be valid,
				// I'm just going to discard the next 14 bytes and hope it'll still work out.
				self.read(&mut [0; 14]);
				None
			}
			2 => self.position_update(),
			_ => {
				self.connected = false;
				None
			}
		}
	}

	fn general_message(&mut self) -> Option<UserEvent> {
		let mut msg_header: [u8; 16] = [0; 16];
		let n = self.read(&mut msg_header)?;
		if n < 16 {
			return None;
		}

		// let id1 = packet.read_i32(0);
		// let id2 = packet.read_i32(4);
		let opcode = msg_header.read_u32(8);
		let size = msg_header.read_u32(12);

		// Would be a bad idea to dynamically allocate a number of bytes that could be u32::MAX.
		if size > 1024 {
			self.connected = false;
			return None;
		}

		let mut packet = [0; 1024];
		let n = self.read(&mut packet[..size as usize])?;
		if n < size as usize {
			return None;
		}

		let event = match opcode {
			0 => self.cmsg_new_user(&packet),
			6 => self.msg_common(&packet),
			7 => self.cmsg_state_change(&packet),
			_ => None,
		}?;

		Some(event)
	}

	fn position_update(&mut self) -> Option<UserEvent> {
		let mut packet: [u8; 26] = [0; 26];
		let n = self.read(&mut packet)?;
		if n < 26 {
			return None;
		}

		self.position.set(
			packet.read_f32(12),
			packet.read_f32(16),
			packet.read_f32(20),
		);

		Some(UserEvent::PositionUpdate(self.position.clone()))
	}

	/* General Message Receivers */

	fn cmsg_new_user(&mut self, packet: &[u8]) -> Option<UserEvent> {
		let username = packet.read_string(0);
		if packet.len() < username.len() + 1 {
			return None;
		}

		let avatar = validate_avatar(packet.read_string(username.len() + 1));

		self.username.clone_from(&username);
		self.avatar.clone_from(&avatar);

		self.send(&ByteWriter::general_message(
			0,
			self.id,
			Opcode::SMsgClientId,
			&self.id.to_be_bytes(),
		));

		self.send(&ByteWriter::general_message(
			self.id,
			self.id,
			Opcode::SMsgUserJoined,
			&ByteWriter::new(8)
				.write_i32(self.id)
				.write_i32(self.id)
				.write_string(&self.avatar)
				.write_string(&self.username)
				.bytes,
		));

		self.send(&ByteWriter::general_message(
			self.id,
			self.id,
			Opcode::SMsgBroadcastId,
			&self.id.to_be_bytes(),
		));

		Some(UserEvent::NewUser(username, avatar))
	}

	fn msg_common(&mut self, packet: &[u8]) -> Option<UserEvent> {
		if packet.len() < 10 {
			return None;
		}

		let id = packet.read_i32(0);
		let msg_type = packet.read_u32(4);
		let strategy = packet[8];
		let content = &packet[9..];

		match msg_type {
			2 => self.transform_update(content),

			9 => self.chat_send(content),

			12 => self.character_update(content),
			13 => self.name_change(content),
			14 => self.avatar_change(content),
			15 => self.private_chat(id, content),

			10000 => self.appl_specific(id, strategy, content),

			_ => None,
		}
	}

	fn cmsg_state_change(&self, _packet: &[u8]) -> Option<UserEvent> {
		Some(UserEvent::StateChange)
	}

	/* Message Common Receivers */

	fn transform_update(&mut self, content: &[u8]) -> Option<UserEvent> {
		if content.len() < 48 {
			return None;
		}

		let mut mat = Mat3::new();
		for i in 0..9 {
			mat.data[i] = content.read_f32(i * 4);
		}
		self.rotation = mat;

		self.position = Vector3::new(
			content.read_f32(36),
			content.read_f32(40),
			content.read_f32(44),
		);

		Some(UserEvent::TransformUpdate(
			self.rotation.clone(),
			self.position.clone(),
		))
	}

	fn chat_send(&self, content: &[u8]) -> Option<UserEvent> {
		let text = content.read_string(0);

		// Don't send empty messages.
		let (_, message) = text.split_once(": ")?;
		if message.is_empty() {
			return None;
		}

		Some(UserEvent::ChatSend(message.to_string()))
	}

	fn character_update(&mut self, content: &[u8]) -> Option<UserEvent> {
		let character_data = content.read_string(0);
		self.data.clone_from(&character_data);

		Some(UserEvent::CharacterUpdate(character_data))
	}

	fn name_change(&mut self, content: &[u8]) -> Option<UserEvent> {
		let name = content.read_string(0);
		self.username.clone_from(&name);

		Some(UserEvent::NameChange(name))
	}

	fn avatar_change(&mut self, content: &[u8]) -> Option<UserEvent> {
		let avatar = validate_avatar(content.read_string(0));
		self.avatar.clone_from(&avatar);

		Some(UserEvent::AvatarChange(avatar))
	}

	fn private_chat(&mut self, id: i32, content: &[u8]) -> Option<UserEvent> {
		if content.len() < 5 {
			return None;
		}
		let text = content.read_string(4);

		Some(UserEvent::PrivateChat(id, text))
	}

	fn appl_specific(&mut self, id: i32, strategy: u8, content: &[u8]) -> Option<UserEvent> {
		if content.len() < 7 {
			return None;
		}

		// Unknown = content.read_u8(0);
		let method = content.read_string(1);
		if content.len() < method.len() + 2 {
			return None;
		}

		let strarg = content.read_string(method.len() + 2);
		if content.len() < method.len() + strarg.len() + 6 {
			return None;
		}

		let intarg = content.read_i32(method.len() + strarg.len() + 3);

		Some(UserEvent::ApplSpecific(
			strategy.into(),
			id,
			method,
			strarg,
			intarg,
		))
	}
}
