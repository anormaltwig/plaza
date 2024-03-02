use std::{
	collections::HashMap,
	io::{self, Read, Write},
	net::{TcpListener, ToSocketAddrs},
	sync::mpsc::{self, Receiver, Sender},
	thread::{self, JoinHandle},
	time::{Duration, SystemTime},
};

use crate::{
	math::{Mat3, Vector3},
	protocol::{ByteWriter, MsgCommon, Opcode},
	user::{User, UserEvent},
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
	#[allow(dead_code)] // Remove when done implementing WLS.
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
	/// Starts a new bureau and returns a speacial handle for its thread.
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

			let now = SystemTime::now();

			if let Ok((socket, _addr)) = self.listener.accept() {
				if let Ok(()) = socket.set_nonblocking(true) {
					connecting.push((now.clone(), socket));
				}
			}

			// Handling pending users.
			let mut i = 0;
			while i < connecting.len() {
				let (connect_time, socket) = &mut connecting[i];

				let mut hello_buf = [0; 7];
				let n = match socket.read(&mut hello_buf) {
					Ok(n) => n,
					Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
						if let Ok(duration) = now.duration_since(*connect_time) {
							if duration.as_secs() > 10 {
								connecting.swap_remove(i);
							} else {
								i += 1;
							}
						} else {
							connecting.swap_remove(i);
						}

						continue;
					}
					Err(_) => {
						connecting.swap_remove(i);
						continue;
					}
				};

				let mut socket = connecting.swap_remove(i).1;

				if n < 7 {
					continue;
				}

				for j in 0..5 {
					if hello_buf[j] != b"hello"[j] {
						continue;
					}
				}
				// Last two bytes are likely browser version, doesn't seem important to check.

				let id = match self.get_available_id() {
					Some(id) => id,
					None => continue,
				};

				let buf = [
					&b"hello\0"[..],
					&id.to_be_bytes()[..],
					&id.to_be_bytes()[..],
				]
				.concat();

				if let Ok(n) = socket.write(&buf) {
					if n != buf.len() {
						continue;
					}
				}

				if let Ok(user) = User::new(socket, id) {
					self.users.insert(id, user);
				}
			}

			// Handle connected users.
			for (_id, user) in &self.users {
				match user.poll() {
					Some(events) => {
						for event in events {
							match event {
								UserEvent::NewUser => self.broadcast_user_count(),
								UserEvent::StateChange => (),
								UserEvent::PositionUpdate(pos) => self.position_update(user, pos),
								UserEvent::TransformUpdate(mat, pos) => {
									self.transform_update(user, mat, pos)
								}
								UserEvent::ChatSend(msg) => self.chat_send(user, msg),
								UserEvent::CharacterUpdate(data) => {
									self.character_update(user, data)
								}
								UserEvent::NameChange(name) => self.name_change(user, name),
								UserEvent::AvatarChange(avatar) => self.avatar_change(user, avatar),
								UserEvent::PrivateChat(receiver, msg) => {
									self.private_chat(user, receiver, msg)
								}
								UserEvent::ApplSpecific(strategy, id, method, strarg, intarg) => {
									self.appl_specific(user, strategy, id, method, strarg, intarg)
								}
							}
						}
					}
					None => (),
				}

				if !user.is_connected() {
					self.disconnect_user(&user);
				}
			}

			let mut removed = 0;
			self.users.retain(|_, user| {
				let connected = user.is_connected();

				if !connected {
					removed += 1;
				}

				connected
			});

			if removed > 0 {
				self.broadcast_user_count();
			}

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

	fn update_aura(&self, user: &User) {
		let user_pos = user.get_pos();

		for (id, other) in &self.users {
			if *id == user.id {
				continue;
			}

			let dist = user_pos.get_distance_sqr(&other.get_pos());

			if user.check_aura(id) {
				if dist > self.options.aura_radius.powi(2) {
					user.remove_aura(&other.id);
					other.remove_aura(&user.id);

					// Tell other that user is gone
					let mut user_left = ByteWriter::new();
					user_left.write_i32(user.id);
					other.send(&ByteWriter::general_message(
						other.id,
						other.id,
						Opcode::SMsgUserLeft,
						&user_left,
					));

					// Tell user that other is gone
					let mut other_left = ByteWriter::new();
					other_left.write_i32(other.id);
					user.send(&ByteWriter::general_message(
						user.id,
						user.id,
						Opcode::SMsgUserLeft,
						&other_left,
					));
				}
			} else if dist <= self.options.aura_radius.powi(2) {
				user.add_aura(other.id);
				other.add_aura(user.id);

				// Send user to other
				let mut user_joined = ByteWriter::new();
				user_joined.write_i32(user.id);
				user_joined.write_i32(user.id);
				user_joined.write_string(&user.get_avatar());
				user_joined.write_string(&user.get_name());

				let mut user_cupdate = ByteWriter::new();
				user_cupdate.write_string(&user.get_data());

				other.send(&ByteWriter::general_message(
					other.id,
					other.id,
					Opcode::SMsgUserJoined,
					&user_joined,
				));
				other.send(&ByteWriter::message_common(
					other.id,
					other.id,
					user.id,
					MsgCommon::CharacterUpdate,
					1,
					&user_cupdate,
				));

				// Send other to user
				let mut other_joined = ByteWriter::new();
				other_joined.write_i32(other.id);
				other_joined.write_i32(other.id);
				other_joined.write_string(&other.get_avatar());
				other_joined.write_string(&other.get_name());

				let mut other_cupdate = ByteWriter::new();
				other_cupdate.write_string(&other.get_data());

				user.send(&ByteWriter::general_message(
					user.id,
					user.id,
					Opcode::SMsgUserJoined,
					&other_joined,
				));
				user.send(&ByteWriter::message_common(
					user.id,
					user.id,
					other.id,
					MsgCommon::CharacterUpdate,
					1,
					&other_cupdate,
				));
			}
		}
	}

	fn send_to_aura(&self, exluded: &User, stream: &ByteWriter) {
		for id in exluded.get_aura() {
			let user = match self.users.get(&id) {
				Some(u) => u,
				None => continue,
			};
			user.send(stream);
		}
	}

	fn send_to_aura_inclusive(&self, user: &User, stream: &ByteWriter) {
		user.send(stream);
		for id in user.get_aura() {
			let other = match self.users.get(&id) {
				Some(u) => u,
				None => continue,
			};
			other.send(stream);
		}
	}

	fn send_to_all(&self, stream: &ByteWriter) {
		for (_id, user) in &self.users {
			user.send(stream);
		}
	}

	fn send_to_others(&self, user: &User, stream: &ByteWriter) {
		for (id, other) in &self.users {
			if *id == user.id {
				continue;
			}

			other.send(stream);
		}
	}

	fn disconnect_user(&self, user: &User) {
		for (id, other) in &self.users {
			if *id == user.id {
				continue;
			}

			if !other.remove_aura(&user.id) {
				continue;
			}

			let mut user_left = ByteWriter::new();
			user_left.write_i32(user.id);
			other.send(&ByteWriter::general_message(
				other.id,
				other.id,
				Opcode::SMsgUserLeft,
				&user_left,
			));
		}
	}

	fn broadcast_user_count(&self) {
		let mut user_count = ByteWriter::new();
		user_count.write_u8(1);
		user_count.write_i32(self.users.len() as i32);

		self.send_to_all(&ByteWriter::general_message(
			0,
			0,
			Opcode::SMsgUserCount,
			&user_count,
		));
	}

	fn position_update(&self, user: &User, pos: Vector3) {
		self.update_aura(user);

		self.send_to_aura(user, &ByteWriter::position_update(user.id, &pos));
	}

	fn transform_update(&self, user: &User, mat: Mat3, pos: Vector3) {
		self.update_aura(user);

		let mut content = ByteWriter::new();
		for i in 0..9 {
			content.write_f32(mat.data[i]);
		}
		content.write_f32(pos.x);
		content.write_f32(pos.y);
		content.write_f32(pos.z);

		self.send_to_aura(
			user,
			&ByteWriter::message_common(
				user.id,
				user.id,
				user.id,
				MsgCommon::TransformUpdate,
				1,
				&content,
			),
		);
	}

	fn chat_send(&self, user: &User, msg: String) {
		let text = format!("{}: {}", user.get_name(), msg).to_string();

		let mut chat_send = ByteWriter::new();
		chat_send.write_string(&text);

		self.send_to_others(
			user,
			&ByteWriter::message_common(
				user.id,
				user.id,
				user.id,
				MsgCommon::ChatSend,
				1,
				&chat_send,
			),
		)
	}

	fn character_update(&self, user: &User, data: String) {
		let mut character_update = ByteWriter::new();
		character_update.write_string(&data);

		self.send_to_aura(
			user,
			&ByteWriter::message_common(
				user.id,
				user.id,
				user.id,
				MsgCommon::CharacterUpdate,
				1,
				&character_update,
			),
		)
	}

	fn name_change(&self, user: &User, name: String) {
		let mut name_change = ByteWriter::new();
		name_change.write_string(&name);

		self.send_to_others(
			user,
			&ByteWriter::message_common(
				user.id,
				user.id,
				user.id,
				MsgCommon::NameChange,
				1,
				&name_change,
			),
		)
	}

	fn avatar_change(&self, user: &User, avatar: String) {
		let mut avatar_change = ByteWriter::new();
		avatar_change.write_string(&avatar);

		self.send_to_others(
			user,
			&ByteWriter::message_common(
				user.id,
				user.id,
				user.id,
				MsgCommon::AvatarChange,
				1,
				&avatar_change,
			),
		)
	}

	fn private_chat(&self, user: &User, receiver: i32, text: String) {
		let other = match self.users.get(&receiver) {
			Some(u) => u,
			None => return,
		};

		let mut pchat = ByteWriter::new();
		pchat.write_i32(user.id);
		pchat.write_string(&text);

		other.send(&ByteWriter::message_common(
			user.id,
			user.id,
			user.id,
			MsgCommon::PrivateChat,
			2,
			&pchat,
		))
	}

	fn appl_specific(
		&self,
		user: &User,
		strategy: u8,
		id: i32,
		method: String,
		strarg: String,
		intarg: i32,
	) {
		let mut appl = ByteWriter::new();
		appl.write_u8(2);
		appl.write_string(&method);
		appl.write_string(&strarg);
		appl.write_i32(intarg);
		let stream = ByteWriter::message_common(
			user.id,
			user.id,
			id,
			MsgCommon::ApplSpecific,
			strategy,
			&appl,
		);

		// This could be wrong... :3c
		if id == -9999 {
			let master = match self.users.iter().next() {
				Some((_id, user)) => user,
				None => return,
			};

			match strategy {
				// Missing two other strategies here, I've yet to figure out what exactly they do.
				// I'm guessing they have to do with the master client responding to a message.
				2 => master.send(&stream),
				_ => (),
			}

			return;
		}

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
