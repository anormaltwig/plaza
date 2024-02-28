use std::{
	collections::HashMap,
	io::{self, Read, Write},
	net::{TcpListener, ToSocketAddrs},
	sync::mpsc::{self, Receiver, Sender},
	thread::{self, JoinHandle},
	time::{Duration, SystemTime},
};

use crate::core::math::{Mat3, Vector3};

use super::{
	protocol::{ByteReader, ByteStream, MsgCommon, Opcode},
	user::User,
};

enum BureauSignal {
	Close,
}

pub struct BureauOptions {
	pub max_players: u16,
	pub aura_radius: f32,
}

pub struct BureauHandle {
	handle: Option<JoinHandle<()>>,
	signaller: Sender<BureauSignal>,
}

impl BureauHandle {
	pub fn close(&mut self) {
		_ = self.signaller.send(BureauSignal::Close);
	}

	pub fn join(mut self) {
		self.handle
			.take()
			.expect("Tried to join invalid bureau.")
			.join()
			.expect("Failed to join thread");
	}
}

pub struct Bureau {
	users: HashMap<i32, User>,
	user_index: u16,

	listener: TcpListener,
	receiver: Receiver<BureauSignal>,
	options: BureauOptions,
}

impl Bureau {
	pub fn new<A>(addr: A, options: BureauOptions) -> io::Result<BureauHandle>
	where
		A: ToSocketAddrs,
	{
		let listener = TcpListener::bind(addr)?;
		listener.set_nonblocking(true)?;

		let (signaller, receiver) = mpsc::channel::<BureauSignal>();

		let bureau = Bureau {
			users: HashMap::new(),
			user_index: 0,

			listener,
			receiver,
			options,
		};

		let handle = thread::spawn(move || bureau.run());

		Ok(BureauHandle {
			handle: Some(handle),
			signaller,
		})
	}

	fn run(mut self) {
		let mut connecting = Vec::new();

		loop {
			if let Ok(signal) = self.receiver.try_recv() {
				match signal {
					BureauSignal::Close => break,
				}
			}

			let now_time = SystemTime::now();

			if let Ok((socket, _addr)) = self.listener.accept() {
				if let Ok(()) = socket.set_nonblocking(true) {
					connecting.push((now_time.clone(), Some(socket)));
				}
			}

			connecting.retain_mut(|(connect_time, opt_socket)| {
				let mut socket = opt_socket.as_ref().expect("This should never be empty.");
				let mut hello_buf = [0; 7];

				match socket.read(&mut hello_buf) {
					Ok(n) => {
						if n < 7 {
							return false;
						}

						for i in 0..5 {
							if hello_buf[i] != b"hello"[i] {
								return false;
							}
						}
						// Last two bytes are likely browser version, doesn't seem important to check.

						if let Some(id) = self.get_available_id() {
							let buf = [
								&b"hello\0"[..],
								&id.to_be_bytes()[..],
								&id.to_be_bytes()[..],
							]
							.concat();

							if let Ok(n) = socket.write(&buf) {
								if n != buf.len() {
									return false;
								}

								if let Ok(user) = User::new(opt_socket.take().unwrap(), id) {
									self.users.insert(id, user);
								}
							}
						}

						false
					}
					Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
						if let Ok(duration) = SystemTime::now().duration_since(*connect_time) {
							if duration.as_secs() < 10 {
								return true;
							}
						}

						false
					}
					Err(_) => false,
				}
			});

			for (_id, user) in &self.users {
				let mut buf: [u8; 512] = [0; 512];
				match user.read(&mut buf) {
					Some(n) => {
						let mut packet = &buf[..n];

						while packet.len() > 17 {
							let res = match packet[0] {
								0 => self.general_message(user, packet),
								// 1 => unknown purpose,
								2 => self.position_update(user, packet),
								_ => break,
							};

							let size = match res {
								Some(s) => s,
								None => break,
							};

							packet = &packet[size..];
						}
					}
					None => (),
				}

				if !user.is_connected() {
					self.disconnect_user(&user);
					self.broadcast_user_count();
				}
			}

			self.users.retain(|_, user| user.is_connected());

			thread::sleep(Duration::from_millis(100));
		}
	}

	fn get_available_id(&mut self) -> Option<i32> {
		// Check values between 1 and max_players inclusive and return the first unused id
		for i in 0..self.options.max_players {
			let id = self.user_index.overflowing_add(i).0 % self.options.max_players + 1;
			if !self.users.contains_key(&(id as i32)) {
				self.user_index = id;
				return Some(id as i32);
			}
		}

		None
	}

	fn send_to_aura(&self, exluded: &User, stream: &ByteStream) {
		for id in exluded.get_aura() {
			let user = match self.users.get(&id) {
				Some(u) => u,
				None => continue,
			};
			user.send(stream);
		}
	}

	fn send_to_aura_inclusive(&self, user: &User, stream: &ByteStream) {
		user.send(stream);
		for id in user.get_aura() {
			let other = match self.users.get(&id) {
				Some(u) => u,
				None => continue,
			};
			other.send(stream);
		}
	}

	fn send_to_all(&self, stream: &ByteStream) {
		for (_id, user) in &self.users {
			user.send(stream);
		}
	}

	fn send_to_others(&self, user: &User, stream: &ByteStream) {
		for (id, other) in &self.users {
			if *id == user.id {
				continue;
			}

			other.send(stream);
		}
	}

	/* Section Types */

	fn general_message(&self, user: &User, packet: &[u8]) -> Option<usize> {
		let id1 = packet.read_i32(1);
		let id2 = packet.read_i32(5);
		let opcode = packet.read_u32(9);
		let size = packet.read_u32(13);

		if packet.len() < 17 + size as usize {
			return None;
		}

		let body = &packet[17..];

		match opcode {
			0 => self.cmsg_new_user(user, body),
			6 => self.msg_common(user, id1, id2, body),
			7 => self.cmsg_state_change(user, body),
			_ => return None,
		}

		Some((17 + size) as usize)
	}

	fn position_update(&self, user: &User, packet: &[u8]) -> Option<usize> {
		if packet.len() < 27 {
			return None;
		}

		user.set_pos(Vector3::new(
			packet.read_f32(13),
			packet.read_f32(17),
			packet.read_f32(21),
		));

		// Update auras
		for (id, other) in &self.users {
			if *id == user.id {
				continue;
			}

			let dist = user.get_pos().get_distance_sqr(&other.get_pos());
			if !other.check_aura(&user.id) {
				if dist > self.options.aura_radius.powi(2) {
					continue;
				}

				// Join Aura
				user.add_aura(other.id);
				let mut user_joined = ByteStream::new();
				user_joined.write_i32(other.id);
				user_joined.write_i32(other.id);
				user_joined.write_string(&other.get_avatar());
				user_joined.write_string(&other.get_name());

				let mut character_update = ByteStream::new();
				character_update.write_string(&other.get_data());

				user.send(&ByteStream::general_message(
					0,
					0,
					Opcode::SMsgUserJoined,
					&user_joined,
				));
				user.send(&ByteStream::message_common(
					0,
					0,
					other.id,
					MsgCommon::CharacterUpdate,
					1,
					&character_update,
				));

				other.add_aura(user.id);
				let mut user_joined = ByteStream::new();
				user_joined.write_i32(user.id);
				user_joined.write_i32(user.id);
				user_joined.write_string(&user.get_avatar());
				user_joined.write_string(&user.get_name());

				let mut character_update = ByteStream::new();
				character_update.write_string(&user.get_data());

				other.send(&ByteStream::general_message(
					0,
					0,
					Opcode::SMsgUserJoined,
					&user_joined,
				));
				other.send(&ByteStream::message_common(
					0,
					0,
					user.id,
					MsgCommon::CharacterUpdate,
					1,
					&character_update,
				));
			} else if dist > self.options.aura_radius.powi(2) {
				// Leave Aura
				user.remove_aura(&other.id);
				let mut user_left = ByteStream::new();
				user_left.write_i32(other.id);
				user.send(&ByteStream::general_message(
					0,
					0,
					Opcode::SMsgUserLeft,
					&user_left,
				));

				other.remove_aura(&user.id);
				let mut user_left = ByteStream::new();
				user_left.write_i32(user.id);
				other.send(&ByteStream::general_message(
					0,
					0,
					Opcode::SMsgUserLeft,
					&user_left,
				));
			}
		}

		self.send_to_aura(user, &ByteStream::position_update(user.id, &user.get_pos()));

		Some(27)
	}

	/* General Message Receivers */

	fn cmsg_new_user(&self, user: &User, body: &[u8]) {
		let username = body.read_string(0);
		if body.len() < username.len() + 1 {
			return;
		}

		let avatar = body.read_string(username.len() + 1);

		user.set_name(username.clone());
		user.set_avatar(avatar.clone());

		// Client Id
		let mut client_id = ByteStream::new();
		client_id.write_i32(user.id);

		let stream = ByteStream::general_message(0, user.id, Opcode::SMsgClientId, &client_id);
		user.send(&stream);

		// Unnamed 1
		let mut unnamed1 = ByteStream::new();
		unnamed1.write_u8(1);

		let stream = ByteStream::general_message(user.id, user.id, Opcode::SMsgUnnamed1, &unnamed1);
		user.send(&stream);

		// User Joined
		let mut user_joined = ByteStream::new();
		user_joined.write_i32(user.id);
		user_joined.write_i32(user.id);
		user_joined.write_string(&user.get_avatar());
		user_joined.write_string(&user.get_name());

		let stream =
			ByteStream::general_message(user.id, user.id, Opcode::SMsgUserJoined, &user_joined);
		user.send(&stream);

		// Broadcast Id
		let mut broadcast_id = ByteStream::new();
		broadcast_id.write_i32(user.id);

		let stream =
			ByteStream::general_message(user.id, user.id, Opcode::SMsgBroadcastId, &broadcast_id);
		user.send(&stream);

		self.broadcast_user_count()
	}

	fn msg_common(&self, user: &User, id1: i32, id2: i32, body: &[u8]) {
		if body.len() < 10 {
			return;
		}

		let id = body.read_i32(0);
		let msg_type = body.read_u32(4);
		let strategy = body[8];
		let content = &body[9..];

		match msg_type {
			2 => self.transform_update(user, content),
			9 => self.chat_send(user, content),
			12 => self.character_update(user, content),
			13 => self.name_change(user, content),
			14 => self.avatar_change(user, content),
			15 => self.private_chat(user, id, content),
			_ => self.appl_specific(user, id, id1, id2, strategy, content),
		}
	}

	fn cmsg_state_change(&self, _user: &User, _body: &[u8]) {}

	/* Message Common Receivers */

	fn transform_update(&self, user: &User, content: &[u8]) {
		if content.len() < 48 {
			return;
		}

		let mut mat = Mat3::new();
		for i in 0..9 {
			mat.data[i] = content.read_f32(i * 4);
		}
		user.set_rot(mat);

		let pos = Vector3::new(
			content.read_f32(36),
			content.read_f32(40),
			content.read_f32(44),
		);
		user.set_pos(pos);

		let mut transform = ByteStream::new();

		let mat = user.get_rot();
		for i in 0..9 {
			transform.write_f32(mat.data[i]);
		}

		let pos = user.get_pos();
		transform.write_f32(pos.x);
		transform.write_f32(pos.y);
		transform.write_f32(pos.z);

		self.send_to_aura(
			user,
			&ByteStream::message_common(
				user.id,
				user.id,
				user.id,
				MsgCommon::TransformUpdate,
				1,
				&transform,
			),
		);
	}

	fn chat_send(&self, user: &User, content: &[u8]) {
		let message = content.read_string(0);

		// Don't send empty messages.
		if message.len() <= user.get_name().len() + 2 {
			return;
		}

		//TODO! Ensure the message uses the name of the user.

		let mut msg = ByteStream::new();
		msg.write_string(&message);

		self.send_to_aura(
			user,
			&ByteStream::message_common(user.id, user.id, user.id, MsgCommon::ChatSend, 1, &msg),
		);
	}

	fn character_update(&self, user: &User, content: &[u8]) {
		let character_data = content.read_string(0);

		let mut msg = ByteStream::new();
		msg.write_string(&character_data);

		user.set_data(character_data);

		self.send_to_aura(
			user,
			&ByteStream::message_common(
				user.id,
				user.id,
				user.id,
				MsgCommon::CharacterUpdate,
				1,
				&msg,
			),
		);
	}

	fn name_change(&self, user: &User, content: &[u8]) {
		let name = content.read_string(0);

		let mut msg = ByteStream::new();
		msg.write_string(&name);

		user.set_name(name);

		self.send_to_aura(
			user,
			&ByteStream::message_common(user.id, user.id, user.id, MsgCommon::NameChange, 1, &msg),
		);
	}

	fn avatar_change(&self, user: &User, content: &[u8]) {
		let avatar = content.read_string(0);

		let mut msg = ByteStream::new();
		msg.write_string(&avatar);

		user.set_avatar(avatar);

		self.send_to_aura(
			user,
			&ByteStream::message_common(
				user.id,
				user.id,
				user.id,
				MsgCommon::AvatarChange,
				1,
				&msg,
			),
		);
	}

	fn private_chat(&self, user: &User, id: i32, content: &[u8]) {
		if content.len() < 5 {
			return;
		}

		let other = match self.users.get(&id) {
			Some(u) => u,
			None => return,
		};

		let mut pchat = ByteStream::new();
		pchat.write_i32(user.id);
		pchat.write_string(&content.read_string(4));

		other.send(&ByteStream::message_common(
			user.id,
			user.id,
			other.id,
			MsgCommon::PrivateChat,
			2,
			&pchat,
		));
	}

	fn appl_specific(
		&self,
		user: &User,
		id: i32,
		id1: i32,
		id2: i32,
		strategy: u8,
		content: &[u8],
	) {
		let len = content.len();
		if len < 2 {
			return;
		}

		let n = content[0];
		let method = content.read_string(1);
		if len < 1 + method.len() + 1 {
			return;
		}

		let strarg = content.read_string(1 + method.len() + 1);
		if len < 1 + method.len() + 1 + strarg.len() + 1 + 4 {
			return;
		}

		let intarg = content.read_i32(1 + method.len() + 1 + strarg.len() + 1);

		println!(
			"[{} -> {} - ({}, {})] {} | {} | {}(\"{}\", {})",
			user.id, id, id1, id2, n, strategy, method, strarg, intarg
		);

		let master = match self.users.iter().next() {
			Some((_id, user)) => user,
			None => return,
		};

		let mut appl = ByteStream::new();
		appl.write_arr(content);
		let stream = ByteStream::message_common(0, 0, id, MsgCommon::ApplSpecific, strategy, &appl);

		// This could be wrong... :3c
		if id == -9999 {
			match strategy {
				// Missing two other strategies here, I've yet to figure out what exactly they do.
				// I'm guessing they have to do with the master client responding to a message.
				2 => master.send(&stream),
				_ => (),
			}
		} else {
			match strategy {
				0 => self.send_to_aura_inclusive(user, &stream),
				1 => self.send_to_aura(user, &stream),
				2 => {
					let target = match self.users.get(&id) {
						Some(u) => u,
						None => return,
					};
					target.send(&stream);
				}
				3 => self.send_to_all(&stream),
				4 => self.send_to_others(user, &stream),
				_ => (),
			}
		}
	}

	// SMsgUserLeft
	fn disconnect_user(&self, user: &User) {
		for (id, other) in &self.users {
			if *id == user.id {
				continue;
			}

			if !other.remove_aura(&user.id) {
				continue;
			}

			let mut user_left = ByteStream::new();
			user_left.write_i32(user.id);
			other.send(&ByteStream::general_message(
				other.id,
				other.id,
				Opcode::SMsgUserLeft,
				&user_left,
			));
		}
	}

	// SMsgUserCount
	fn broadcast_user_count(&self) {
		let mut user_count = ByteStream::new();
		user_count.write_u8(1);
		user_count.write_i32(self.users.len() as i32);

		self.send_to_all(&ByteStream::general_message(
			0,
			0,
			Opcode::SMsgUserCount,
			&user_count,
		));
	}
}
