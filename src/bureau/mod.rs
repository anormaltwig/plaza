use std::{io, net::ToSocketAddrs, thread, time::Duration};

use listener::{Listener, ListenerEvent};
use lua_api::LuaApi;
use math::{Mat3, Vector3};
use protocol::{ByteWriter, MsgCommon, Strategy};
use user::UserEvent;
use user_list::{AwesomeCell, UserList};

mod error;
mod listener;
mod lua_api;
mod math;
mod protocol;
mod user;
mod user_list;

pub use error::*;

#[derive(Clone)]
pub struct BureauConfig {
	pub connect_timeout: u64,
	pub max_users: i32,
	pub max_queue: usize,
	pub aura_radius: f32,
	pub wrl: Option<String>,
}

pub struct Bureau {
	port: u16,
	config: BureauConfig,
	listener: Listener,
	user_list: AwesomeCell<UserList>,
	lua_api: LuaApi,
}

#[allow(unused_mut)]
#[allow(dropping_references)]
impl Bureau {
	pub fn new<A: ToSocketAddrs>(addr: A, config: BureauConfig) -> self::Result<Self> {
		assert!(
			config.max_users > 0,
			"max_users config option wasn't positive ({})",
			config.max_users
		);

		let user_list = AwesomeCell::new(UserList::new(config.max_users));
		let lua_api = LuaApi::new(user_list.clone(), &config)?;
		let listener = Listener::new(addr, config.connect_timeout, config.max_queue)?;

		Ok(Self {
			port: listener.port(),
			listener,
			config,

			user_list,

			lua_api,
		})
	}

	pub fn port(&self) -> u16 {
		self.port
	}

	pub fn config(&self) -> &BureauConfig {
		&self.config
	}

	pub fn user_count(&self) -> usize {
		self.user_list.get().users.len()
	}

	pub fn run(&mut self) -> ! {
		loop {
			self.poll().expect("error during poll");
			thread::sleep(Duration::from_millis(100));
		}
	}

	pub fn poll(&mut self) -> io::Result<()> {
		if let Some(event) = self.listener.poll_event()? {
			match event {
				ListenerEvent::Incoming(addr) => {
					if !self.lua_api.user_connect(addr) {
						self.listener.deny_last();
					}
				}
				ListenerEvent::Accepted(stream) => {
					let mut user_list = self.user_list.get_mut();
					user_list.add(stream)?;
					user_list.send_user_count();
				}
			}
		}

		let ids = self
			.user_list
			.get()
			.users
			.keys()
			.copied()
			.collect::<Vec<_>>();
		for id in ids.iter().copied() {
			let mut user_list = self.user_list.get_mut();
			let user = user_list.users.get_mut(&id).unwrap();
			let Some(event) = user.poll() else {
				continue;
			};
			drop(user_list);

			match event {
				UserEvent::NewUser(username, avatar) => self.new_user(id, username, avatar),
				UserEvent::StateChange => (), // useless
				UserEvent::PositionUpdate(pos) => self.position_update(id, pos),
				UserEvent::TransformUpdate(mat, pos) => self.transform_update(id, mat, pos),
				UserEvent::ChatSend(msg) => self.chat_send(id, msg),
				UserEvent::CharacterUpdate(data) => self.character_update(id, data),
				UserEvent::NameChange(name) => self.name_change(id, name),
				UserEvent::AvatarChange(avatar) => self.avatar_change(id, avatar),
				UserEvent::PrivateChat(receiver, msg) => self.private_chat(id, receiver, msg),
				UserEvent::ApplSpecific(strategy, id2, method, strarg, intarg) => {
					self.appl_specific(id, strategy, id2, method, strarg, intarg)
				}
			}
		}

		self.lua_api.think();

		let count = self.user_list.get().users.len();

		for id in ids {
			let mut user_list = self.user_list.get_mut();
			let user = user_list.users.get(&id).unwrap();
			if !user.connected() {
				user_list.disconnect(id);
				drop(user_list); // lua needs the UserList now

				self.lua_api.user_disconnect(id);
			}
		}

		let mut user_list = self.user_list.get_mut();
		if count != user_list.users.len() {
			user_list.send_user_count();
		}

		Ok(())
	}

	fn update_aura(&mut self, id: i32) {
		let mut user_list = self.user_list.get_mut();
		user_list.for_others(id, |user, other| {
			if !other.initialized() {
				return;
			}

			let in_radius = user.pos().distance_sqr(other.pos()) <= self.config.aura_radius.powi(2);
			let in_aura = user.aura().contains(&other.id());

			if !in_aura && in_radius {
				user.add_aura(other);
				other.add_aura(user);
			} else if in_aura && !in_radius {
				user.remove_aura(other);
				other.remove_aura(user);
			}
		});
	}

	fn new_user(&mut self, id: i32, username: String, avatar: String) {
		let mut user_list = self.user_list.get_mut();
		user_list.master();
		user_list.send_user_count();

		let ip = user_list.users.get_mut(&id).unwrap().addr().ip();
		drop(user_list);

		self.lua_api.new_user(id, &username, &avatar, ip);
	}

	fn position_update(&mut self, id: i32, pos: Vector3) {
		self.update_aura(id);
		self.user_list
			.get_mut()
			.send_aura(id, &ByteWriter::position_update(id, &pos).bytes);

		self.lua_api.pos_update(id, &pos);
	}

	fn transform_update(&mut self, id: i32, rot: Mat3, pos: Vector3) {
		self.update_aura(id);

		let mut transform_update = ByteWriter::new(12 * 4);

		for f in rot.data {
			transform_update = transform_update.write_f32(f);
		}

		transform_update = transform_update
			.write_f32(pos.x)
			.write_f32(pos.y)
			.write_f32(pos.z);

		self.user_list.get_mut().send_aura(
			id,
			&ByteWriter::message_common(
				id,
				id,
				MsgCommon::TransformUpdate,
				Strategy::AuraClients,
				&transform_update.bytes,
			)
			.bytes,
		);

		self.lua_api.trans_update(id, &rot);
	}

	fn chat_send(&mut self, id: i32, mut msg: String) {
		if let Some(new_msg) = self.lua_api.chat_send(id, &msg) {
			if new_msg.is_empty() {
				return;
			}

			msg = new_msg;
		}

		let mut user_list = self.user_list.get_mut();

		let text = format!("{}: {}", user_list.users.get(&id).unwrap().username(), msg);

		user_list.send_aura(
			id,
			&ByteWriter::message_common(
				id,
				id,
				MsgCommon::ChatSend,
				Strategy::AllClientsExceptSender,
				&ByteWriter::new(text.len() + 1).write_string(&text).bytes,
			)
			.bytes,
		);
	}

	fn character_update(&mut self, id: i32, data: String) {
		self.user_list.get_mut().send_aura(
			id,
			&ByteWriter::message_common(
				id,
				id,
				MsgCommon::CharacterUpdate,
				Strategy::AuraClientsExceptSender,
				&ByteWriter::new(data.len() + 1).write_string(&data).bytes,
			)
			.bytes,
		);
	}

	fn name_change(&mut self, id: i32, name: String) {
		self.user_list.get_mut().send_aura(
			id,
			&ByteWriter::message_common(
				id,
				id,
				MsgCommon::NameChange,
				Strategy::AuraClientsExceptSender,
				&ByteWriter::new(name.len() + 1).write_string(&name).bytes,
			)
			.bytes,
		);

		self.lua_api.name_change(id, &name);
	}

	fn avatar_change(&mut self, id: i32, avatar: String) {
		self.user_list.get_mut().send_aura(
			id,
			&ByteWriter::message_common(
				id,
				id,
				MsgCommon::AvatarChange,
				Strategy::AuraClientsExceptSender,
				&ByteWriter::new(avatar.len() + 1)
					.write_string(&avatar)
					.bytes,
			)
			.bytes,
		);

		self.lua_api.avatar_change(id, &avatar);
	}

	fn private_chat(&mut self, id: i32, receiver: i32, mut text: String) {
		let is_special = matches!(
			text.as_str(),
			"%%REQ" | "%%RINGING" | "%%REJECT" | "%%ACCEPT" | "%%OK" | "%%BUSY" | "%%END"
		);

		if !is_special {
			let Some((_, msg)) = text.split_once(": ") else {
				return;
			};

			if msg.is_empty() {
				return;
			}

			let content = match self.lua_api.private_chat(id, receiver, msg) {
				Some(new_msg) => {
					if new_msg.is_empty() {
						return;
					}

					new_msg
				}
				None => msg.to_string(),
			};

			text = format!(
				"{}: {}",
				self.user_list.get().users.get(&id).unwrap().username(),
				content
			);
		}

		let mut user_list = self.user_list.get_mut();
		let Some(other) = user_list.users.get_mut(&receiver) else {
			return;
		};

		other.send(
			&ByteWriter::message_common(
				id,
				id,
				MsgCommon::PrivateChat,
				Strategy::SpecificClient,
				&ByteWriter::new(4).write_i32(id).write_string(&text).bytes,
			)
			.bytes,
		)
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
		let writer = ByteWriter::message_common(
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

		let mut user_list = self.user_list.get_mut();

		if id2 == -9999 {
			match strategy {
				// This could be wrong... :3c
				Strategy::AuraClients | Strategy::AllClients | Strategy::Unknown5 => {
					user_list.send_all(&writer.bytes)
				}
				Strategy::AuraClientsExceptSender
				| Strategy::AllClientsExceptSender
				| Strategy::Unknown6 => user_list.send_others(id, &writer.bytes),
				Strategy::SpecificClient => {
					let master_id = match user_list.master() {
						Some(master_id) => master_id,
						None => return,
					};

					let Some(user) = user_list.users.get_mut(&master_id) else {
						return;
					};

					user.send(&writer.bytes);
				}
				Strategy::Invalid => (),
			}

			return;
		}

		match strategy {
			Strategy::AuraClients => {
				user_list.send_aura(id, &writer.bytes);
				user_list.users.get_mut(&id).unwrap().send(&writer.bytes);
			}
			Strategy::AuraClientsExceptSender => user_list.send_aura(id, &writer.bytes),
			Strategy::SpecificClient => {
				let Some(target) = user_list.users.get_mut(&id2) else {
					return;
				};

				target.send(&writer.bytes);
			}
			Strategy::AllClients => user_list.send_all(&writer.bytes),
			Strategy::AllClientsExceptSender => user_list.send_others(id, &writer.bytes),
			_ => (),
		}
	}
}
