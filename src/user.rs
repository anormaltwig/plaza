use std::{
	cell::RefCell,
	collections::HashSet,
	io::{self, ErrorKind, Read, Write},
	net::{SocketAddr, TcpStream},
};

use crate::{
	math::{Mat3, Vector3},
	protocol::{ByteReader, ByteWriter, MsgCommon, Opcode},
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
	ApplSpecific(u8, i32, String, String, i32),
}

struct UserInternal {
	socket: TcpStream,
	connected: bool,
	aura: HashSet<i32>,
	position: Vector3,
	rotation: Mat3,
	username: String,
	avatar: String,
	data: String,
}

pub struct User {
	internal: RefCell<UserInternal>,
	pub id: i32,
}

impl User {
	/// Creates a new user from a TcpStream and i32 id.
	pub fn new(socket: TcpStream, id: i32) -> io::Result<Self> {
		socket.set_nonblocking(true)?;

		Ok(Self {
			internal: RefCell::new(UserInternal {
				socket,
				connected: true,
				aura: HashSet::new(),
				position: Vector3::new(0.0, 0.0, 0.0),
				rotation: Mat3::new(),
				username: String::new(),
				avatar: String::new(),
				data: String::new(),
			}),
			id,
		})
	}

	pub fn is_connected(&self) -> bool {
		self.internal.borrow().connected
	}

	pub fn disconnect(&self) {
		self.internal.borrow_mut().connected = false;
	}

	pub fn peer_addr(&self) -> io::Result<SocketAddr> {
		Ok(self.internal.borrow().socket.peer_addr()?)
	}

	pub fn set_pos(&self, pos: &Vector3) {
		self.internal.borrow_mut().position.set(pos.x, pos.y, pos.z);
		self.send(&ByteWriter::position_update(self.id, pos));
	}
	pub fn pos(&self) -> Vector3 {
		self.internal.borrow().position.clone()
	}

	/// Running this function before a user has chance to send their position will cause them to go to the world origin.
	pub fn set_rot(&self, rot: Mat3) {
		let mut content = ByteWriter::new();
		for i in 0..9 {
			content = content.write_f32(rot.data[i]);
		}

		let mut internal = self.internal.borrow_mut();
		internal.rotation = rot;
		content = content
			.write_f32(internal.position.x)
			.write_f32(internal.position.y)
			.write_f32(internal.position.z);
		drop(internal);

		self.send(&ByteWriter::message_common(
			self.id,
			self.id,
			self.id,
			MsgCommon::TransformUpdate,
			2,
			&content,
		));
	}
	pub fn rot(&self) -> Mat3 {
		self.internal.borrow().rotation.clone()
	}

	pub fn set_name(&self, name: String) {
		self.internal.borrow_mut().username = name
	}
	pub fn name(&self) -> String {
		self.internal.borrow().username.clone()
	}

	pub fn set_avatar(&self, avatar: String) {
		self.internal.borrow_mut().avatar = avatar
	}
	pub fn avatar(&self) -> String {
		self.internal.borrow().avatar.clone()
	}

	pub fn set_data(&self, data: String) {
		self.internal.borrow_mut().data = data;
	}
	pub fn data(&self) -> String {
		self.internal.borrow().data.clone()
	}

	pub fn add_aura(&self, id: i32) -> bool {
		self.internal.borrow_mut().aura.insert(id)
	}
	/// Returns true if the user with the specified is currently in this users aura.
	pub fn check_aura(&self, id: &i32) -> bool {
		self.internal.borrow().aura.contains(id)
	}
	pub fn remove_aura(&self, id: &i32) -> bool {
		self.internal.borrow_mut().aura.remove(id)
	}
	pub fn aura(&self) -> HashSet<i32> {
		self.internal.borrow_mut().aura.clone()
	}

	pub fn send_msg(&self, msg: &str) {
		self.send(&ByteWriter::message_common(
			self.id,
			self.id,
			self.id,
			MsgCommon::ChatSend,
			0,
			&ByteWriter::new().write_string(msg),
		));
	}

	pub fn send(&self, stream: &ByteWriter) {
		let mut internal = self.internal.borrow_mut();

		if let Err(_) = internal.socket.write_all(&stream.bytes) {
			internal.connected = false;
		}
	}

	pub fn poll(&self) -> Option<Vec<UserEvent>> {
		let mut internal = self.internal.borrow_mut();

		let mut buf: [u8; 512] = [0; 512];
		let n = match internal.socket.read(&mut buf) {
			Ok(n) => {
				if n < 17 {
					if n == 0 {
						internal.connected = false;
					}
					return None;
				}

				n
			}
			Err(e) if e.kind() == ErrorKind::WouldBlock => return None,
			Err(_) => {
				internal.connected = false;
				return None;
			}
		};

		drop(internal);

		let mut events = Vec::new();
		let mut packet = &buf[..n];

		while packet.len() > 17 {
			let res = match packet[0] {
				0 => self.general_message(packet),
				// 1 => unknown purpose,
				2 => self.position_update(packet),
				_ => break,
			};

			let (size, event) = match res {
				Some(v) => v,
				None => break,
			};

			events.push(event);
			packet = &packet[size..];
		}

		Some(events)
	}

	fn general_message(&self, packet: &[u8]) -> Option<(usize, UserEvent)> {
		// let id1 = packet.read_i32(1);
		// let id2 = packet.read_i32(5);
		let opcode = packet.read_u32(9);
		let size = packet.read_u32(13);

		if packet.len() < 17 + size as usize {
			return None;
		}

		let body = &packet[17..];

		let event = match opcode {
			0 => self.cmsg_new_user(body),
			6 => self.msg_common(body),
			7 => self.cmsg_state_change(body),
			_ => None,
		}?;

		Some(((17 + size) as usize, event))
	}

	fn position_update(&self, packet: &[u8]) -> Option<(usize, UserEvent)> {
		if packet.len() < 27 {
			return None;
		}

		self.internal.borrow_mut().position.set(
			packet.read_f32(13),
			packet.read_f32(17),
			packet.read_f32(21),
		);

		Some((27, UserEvent::PositionUpdate(self.pos())))
	}

	/* General Message Receivers */

	fn cmsg_new_user(&self, body: &[u8]) -> Option<UserEvent> {
		let username = body.read_string(0);
		if body.len() < username.len() + 1 {
			return None;
		}

		let avatar = body.read_string(username.len() + 1);

		self.set_name(username.clone());
		self.set_avatar(avatar.clone());

		self.send(&ByteWriter::general_message(
			0,
			self.id,
			Opcode::SMsgClientId,
			&ByteWriter::new().write_i32(self.id),
		));

		self.send(&ByteWriter::general_message(
			self.id,
			self.id,
			Opcode::SMsgUserJoined,
			&ByteWriter::new()
				.write_i32(self.id)
				.write_i32(self.id)
				.write_string(&self.avatar())
				.write_string(&self.name()),
		));

		self.send(&ByteWriter::general_message(
			self.id,
			self.id,
			Opcode::SMsgBroadcastId,
			&ByteWriter::new().write_i32(self.id),
		));

		Some(UserEvent::NewUser(username, avatar))
	}

	fn msg_common(&self, body: &[u8]) -> Option<UserEvent> {
		if body.len() < 10 {
			return None;
		}

		let id = body.read_i32(0);
		let msg_type = body.read_u32(4);
		let strategy = body[8];
		let content = &body[9..];

		match msg_type {
			2 => self.transform_update(content),
			9 => self.chat_send(content),
			12 => self.character_update(content),
			13 => self.name_change(content),
			14 => self.avatar_change(content),
			15 => self.private_chat(id, content),

			// Unknown or useless
			3..=8 => None,
			16..=19 => None,

			_ => self.appl_specific(id, strategy, content),
		}
	}

	fn cmsg_state_change(&self, _body: &[u8]) -> Option<UserEvent> {
		Some(UserEvent::StateChange)
	}

	/* Message Common Receivers */

	fn transform_update(&self, content: &[u8]) -> Option<UserEvent> {
		let mut internal = self.internal.borrow_mut();

		if content.len() < 48 {
			return None;
		}

		let mut mat = Mat3::new();
		for i in 0..9 {
			mat.data[i] = content.read_f32(i * 4);
		}
		internal.rotation = mat;

		internal.position = Vector3::new(
			content.read_f32(36),
			content.read_f32(40),
			content.read_f32(44),
		);

		Some(UserEvent::TransformUpdate(
			internal.rotation.clone(),
			internal.position.clone(),
		))
	}

	fn chat_send(&self, content: &[u8]) -> Option<UserEvent> {
		let text = content.read_string(0);

		// Don't send empty messages.
		let (_name, message) = text.split_once(": ")?;
		if message.len() == 0 {
			return None;
		}

		Some(UserEvent::ChatSend(message.to_string()))
	}

	fn character_update(&self, content: &[u8]) -> Option<UserEvent> {
		let character_data = content.read_string(0);

		self.set_data(character_data.clone());

		Some(UserEvent::CharacterUpdate(character_data))
	}

	fn name_change(&self, content: &[u8]) -> Option<UserEvent> {
		let name = content.read_string(0);

		self.set_name(name.clone());

		Some(UserEvent::NameChange(name))
	}

	fn avatar_change(&self, content: &[u8]) -> Option<UserEvent> {
		let avatar = content.read_string(0);

		self.set_avatar(avatar.clone());

		Some(UserEvent::AvatarChange(avatar))
	}

	fn private_chat(&self, id: i32, content: &[u8]) -> Option<UserEvent> {
		if content.len() < 5 {
			return None;
		}
		let text = content.read_string(4);

		Some(UserEvent::PrivateChat(id, text))
	}

	fn appl_specific(&self, id: i32, strategy: u8, content: &[u8]) -> Option<UserEvent> {
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
			strategy, id, method, strarg, intarg,
		))
	}
}
