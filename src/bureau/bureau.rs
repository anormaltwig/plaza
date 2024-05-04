use std::{
	io::{self, ErrorKind, Read},
	net::{SocketAddr, TcpListener, TcpStream},
	thread,
	time::{Duration, Instant},
};

use super::{
	math::{Mat3, Vector3},
	protocol::{ByteWriter, MsgCommon, Opcode, Strategy},
	user::{User, UserEvent},
	user_list::UserList,
};

#[derive(Clone, Copy)]
pub struct BureauOptions {
	pub max_players: i32,
	pub aura_radius: f32,
}

pub struct Bureau {
	pub user_list: UserList,
	pub options: BureauOptions,

	port: u16,
	listener: TcpListener,
	connecting: Vec<(Instant, Option<TcpStream>)>,
}

impl Bureau {
	pub fn new(addr: SocketAddr, options: BureauOptions) -> io::Result<Self> {
		let listener = TcpListener::bind(addr)?;
		listener.set_nonblocking(true)?;

		Ok(Self {
			user_list: UserList::new(options.max_players),
			options,

			port: listener.local_addr()?.port(),
			listener,
			connecting: Vec::new(),
		})
	}

	pub fn port(&self) -> u16 {
		self.port
	}

	pub fn run(&mut self) -> ! {
		loop {
			self.poll();
			thread::sleep(Duration::from_millis(100));
		}
	}

	pub fn poll(&mut self) {
		if let Ok((socket, _)) = self.listener.accept() {
			if let Ok(()) = socket.set_nonblocking(true) {
				self.connecting.push((Instant::now(), Some(socket)));
			}
		}

		for (connect_time, socket) in &mut self.connecting {
			let mut hello_buf = [0; 7];
			let n = match socket.as_mut().unwrap().read(&mut hello_buf) {
				Ok(n) => n,
				Err(e) if e.kind() == ErrorKind::WouldBlock => {
					if connect_time.elapsed().as_secs() > 10 {
						socket.take();
					}
					continue;
				}
				Err(_) => {
					socket.take();
					continue;
				}
			};

			let socket = socket.take().unwrap();

			if n < 7 {
				continue;
			}

			for (i, b) in hello_buf.iter().enumerate() {
				// Last two bytes are vscp version.
				if *b != b"hello\x01\x01"[i] {
					continue;
				}
			}

			if self.user_list.add(socket) {
				self.user_list.send_user_count();
			}
		}

		self.connecting.retain(|(_, socket)| socket.is_some());

		for id in self.user_list.keys().copied().collect::<Vec<i32>>() {
			let user = self.user_list.get_mut(&id).unwrap();

			if let Some(event) = user.poll() {
				self.handle_event(id, event);
			}

			let user = self.user_list.get(&id).unwrap();
			if !user.connected {
				self.disconnect_user(id);
			}
		}

		let mut removed = false;

		self.user_list.retain(|_, user| {
			removed |= !user.connected;
			user.connected
		});

		if removed {
			self.user_list.send_user_count();
		}
	}

	fn send_to_aura(&mut self, id: i32, stream: &ByteWriter) {
		let sender = self.user_list.get(&id).unwrap() as *const User;
		unsafe {
			for id in &(*sender).aura {
				let other = match self.user_list.get_mut(id) {
					Some(user) => user as *mut User,
					None => continue,
				};
				assert_ne!(sender, other, "Tried to borrow the same user twice.");

				(*other).send(stream);
			}
		}
	}

	fn send_to_all(&mut self, stream: &ByteWriter) {
		for (_, user) in self.user_list.iter_mut() {
			user.send(stream);
		}
	}

	fn send_to_other(&mut self, id: i32, stream: &ByteWriter) {
		self.user_list.for_others(&id, |_, other| {
			other.send(stream);
		});
	}

	fn update_aura(&mut self, id: &i32) {
		self.user_list.for_others(id, |user, other| {
			let in_radius =
				user.pos().distance_sqr(other.pos()) <= self.options.aura_radius.powi(2);
			let in_aura = user.aura.contains(&other.id);

			if !in_radius && in_aura {
				other.aura.remove(&user.id);
				other.send(&ByteWriter::general_message(
					other.id,
					user.id,
					Opcode::SMsgUserLeft,
					&ByteWriter::new(4).write_i32(user.id).bytes,
				));

				user.aura.remove(&other.id);
				user.send(&ByteWriter::general_message(
					user.id,
					other.id,
					Opcode::SMsgUserLeft,
					&ByteWriter::new(4).write_i32(other.id).bytes,
				));
			} else if in_radius && !in_aura {
				other.aura.insert(user.id);
				other.send(&ByteWriter::general_message(
					other.id,
					other.id,
					Opcode::SMsgUserJoined,
					&ByteWriter::new(8)
						.write_i32(user.id)
						.write_i32(user.id)
						.write_string(&user.avatar)
						.write_string(&user.username)
						.bytes,
				));
				other.send(&ByteWriter::message_common(
					other.id,
					user.id,
					MsgCommon::CharacterUpdate,
					Strategy::AuraClientsExceptSender,
					&ByteWriter::new(0).write_string(&user.data).bytes,
				));

				user.aura.insert(other.id);
				user.send(&ByteWriter::general_message(
					user.id,
					other.id,
					Opcode::SMsgUserJoined,
					&ByteWriter::new(8)
						.write_i32(other.id)
						.write_i32(other.id)
						.write_string(&other.avatar)
						.write_string(&other.username)
						.bytes,
				));
				user.send(&ByteWriter::message_common(
					user.id,
					other.id,
					MsgCommon::CharacterUpdate,
					Strategy::AuraClientsExceptSender,
					&ByteWriter::new(0).write_string(&other.data).bytes,
				));
			}
		});
	}

	fn disconnect_user(&mut self, id: i32) {
		self.send_to_aura(
			id,
			&ByteWriter::general_message(
				id,
				id,
				Opcode::SMsgUserLeft,
				&ByteWriter::new(4).write_i32(id).bytes,
			),
		);
	}

	fn handle_event(&mut self, id: i32, event: UserEvent) {
		match event {
			UserEvent::NewUser => self.new_user(),
			UserEvent::StateChange => (),
			UserEvent::PositionUpdate(pos) => self.position_update(id, pos),
			UserEvent::TransformUpdate(b) => {
				let (mat, pos) = *b;
				self.transform_update(id, mat, pos)
			}
			UserEvent::ChatSend(msg) => self.chat_send(id, msg),
			UserEvent::CharacterUpdate(data) => self.character_update(id, data),
			UserEvent::NameChange(name) => self.name_change(id, name),
			UserEvent::AvatarChange(avatar) => self.avatar_change(id, avatar),
			UserEvent::PrivateChat(receiver, msg) => self.private_chat(id, receiver, msg),
			UserEvent::ApplSpecific(b) => {
				let (strategy, id2, method, strarg, intarg) = *b;
				self.appl_specific(id, strategy, id2, method, strarg, intarg)
			}
		}
	}

	fn new_user(&mut self) {
		self.user_list.master();
		self.user_list.send_user_count();
	}

	fn position_update(&mut self, id: i32, pos: Vector3) {
		self.update_aura(&id);

		self.send_to_aura(id, &ByteWriter::position_update(id, &pos))
	}

	fn transform_update(&mut self, id: i32, rot: Mat3, pos: Vector3) {
		self.update_aura(&id);

		let mut transform_update = ByteWriter::new(12 * 4);

		for f in rot.data {
			transform_update = transform_update.write_f32(f);
		}

		transform_update = transform_update
			.write_f32(pos.x)
			.write_f32(pos.y)
			.write_f32(pos.z);

		self.send_to_aura(
			id,
			&ByteWriter::message_common(
				id,
				id,
				MsgCommon::TransformUpdate,
				Strategy::AuraClients,
				&transform_update.bytes,
			),
		)
	}

	fn chat_send(&mut self, id: i32, msg: String) {
		let text = format!("{}: {}", self.user_list.get(&id).unwrap().username, msg);

		self.send_to_other(
			id,
			&ByteWriter::message_common(
				id,
				id,
				MsgCommon::ChatSend,
				Strategy::AllClientsExceptSender,
				&ByteWriter::new(text.len() + 1).write_string(&text).bytes,
			),
		);
	}

	fn character_update(&mut self, id: i32, data: String) {
		self.send_to_aura(
			id,
			&ByteWriter::message_common(
				id,
				id,
				MsgCommon::CharacterUpdate,
				Strategy::AuraClientsExceptSender,
				&ByteWriter::new(data.len() + 1).write_string(&data).bytes,
			),
		);
	}

	fn name_change(&mut self, id: i32, name: String) {
		self.send_to_aura(
			id,
			&ByteWriter::message_common(
				id,
				id,
				MsgCommon::NameChange,
				Strategy::AuraClientsExceptSender,
				&ByteWriter::new(name.len() + 1).write_string(&name).bytes,
			),
		);
	}

	fn avatar_change(&mut self, id: i32, avatar: String) {
		self.send_to_aura(
			id,
			&ByteWriter::message_common(
				id,
				id,
				MsgCommon::AvatarChange,
				Strategy::AuraClientsExceptSender,
				&ByteWriter::new(avatar.len() + 1)
					.write_string(&avatar)
					.bytes,
			),
		);
	}

	fn private_chat(&mut self, id: i32, receiver: i32, mut text: String) {
		let user = self.user_list.get(&id).unwrap();

		let is_special = matches!(
			text.as_str(),
			"%%REQ" | "%%RINGING" | "%%REJECT" | "%%ACCEPT" | "%%OK" | "%%BUSY" | "%%END"
		);

		if !is_special {
			let msg = match text.split_once(": ") {
				Some((_name, message)) => {
					if message.is_empty() {
						return;
					}
					message
				}
				None => return,
			};

			text = format!("{}: {}", user.username, msg).to_string();
		}

		let other = match self.user_list.get_mut(&receiver) {
			Some(u) => u,
			None => return,
		};

		other.send(&ByteWriter::message_common(
			id,
			id,
			MsgCommon::PrivateChat,
			Strategy::SpecificClient,
			&ByteWriter::new(0).write_i32(id).write_string(&text).bytes,
		))
	}

	fn appl_specific(
		&mut self,
		id: i32,
		strategy: Strategy,
		id2: i32,
		method: String,
		strarg: String,
		intarg: i32,
	) {
		let stream = ByteWriter::message_common(
			id,
			id2,
			MsgCommon::ApplSpecific,
			strategy,
			&ByteWriter::new(1)
				.write_u8(2)
				.write_string(&method)
				.write_string(&strarg)
				.write_i32(intarg)
				.bytes,
		);

		if id2 == -9999 {
			match strategy {
				// This could be wrong... :3c
				Strategy::AuraClients | Strategy::AllClients | Strategy::Unknown5 => {
					self.send_to_all(&stream)
				}
				Strategy::AuraClientsExceptSender
				| Strategy::AllClientsExceptSender
				| Strategy::Unknown6 => self.send_to_other(id, &stream),
				Strategy::SpecificClient => {
					let master_id = match self.user_list.master() {
						Some(master_id) => master_id,
						None => return,
					};

					let Some(user) = self.user_list.get_mut(&master_id) else {
						return;
					};

					user.send(&stream);
				}
				Strategy::Invalid => (),
			}

			return;
		}

		match strategy {
			Strategy::AuraClients => {
				self.send_to_aura(id, &stream);
				self.user_list.get_mut(&id).unwrap().send(&stream);
			}
			Strategy::AuraClientsExceptSender => self.send_to_aura(id, &stream),
			Strategy::SpecificClient => {
				let Some(target) = self.user_list.get_mut(&id2) else {
					return;
				};

				target.send(&stream);
			}
			Strategy::AllClients => self.send_to_all(&stream),
			Strategy::AllClientsExceptSender => self.send_to_other(id, &stream),
			_ => (),
		}
	}
}
