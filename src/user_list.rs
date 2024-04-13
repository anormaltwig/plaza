use std::{cell::RefCell, collections::HashMap, io::Write, net::TcpStream};

use crate::{
	protocol::{ByteWriter, Opcode},
	user::User,
};

pub struct UserList {
	pub users: HashMap<i32, User>,

	user_index: i32,
	master_user: RefCell<i32>,
	max_players: i32,
}

impl UserList {
	pub fn new(max_players: i32) -> Self {
		Self {
			users: HashMap::new(),

			user_index: 0,
			master_user: RefCell::new(-1),
			max_players: max_players.clamp(1, 2 ^ 31 - 1),
		}
	}

	pub fn new_user(&mut self, mut socket: TcpStream) {
		let id = match self.available_id() {
			Some(id) => id,
			None => return,
		};

		let buf = [
			&b"hello\0"[..],
			&id.to_be_bytes()[..],
			&id.to_be_bytes()[..],
		]
		.concat();

		if let Ok(n) = socket.write(&buf) {
			if n != buf.len() {
				return;
			}
		}

		if let Ok(user) = User::new(socket, id) {
			self.users.insert(id, user);
		}
	}

	pub fn master(&self) -> Option<&User> {
		let mut master_user = self.master_user.borrow_mut();

		if let Some(master) = self.users.get(&master_user) {
			return Some(master);
		}

		*master_user = *self.users.iter().next()?.0;
		let master = self.users.get(&master_user)?;

		master.send(&ByteWriter::general_message(
			master.id,
			master.id,
			Opcode::SMsgSetMaster,
			&ByteWriter::new().write_u8(1),
		));

		Some(master)
	}

	fn available_id(&mut self) -> Option<i32> {
		// Check values between 1 and max_players inclusive and return the first unused id
		for i in 0..self.max_players {
			let id = (self.user_index + i) % self.max_players + 1;
			if !self.users.contains_key(&id) {
				self.user_index = id;
				return Some(id);
			}
		}

		None
	}
}
