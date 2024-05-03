use std::{
	collections::HashMap,
	io::Write,
	net::TcpStream,
	ops::{Deref, DerefMut},
};

use super::{
	protocol::{ByteWriter, Opcode},
	user::User,
};

pub struct UserList {
	users: HashMap<i32, User>,
	max_index: i32,
	user_index: i32,
	master_index: i32,
}

impl Deref for UserList {
	type Target = HashMap<i32, User>;

	fn deref(&self) -> &Self::Target {
		&self.users
	}
}

impl DerefMut for UserList {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.users
	}
}

impl UserList {
	pub fn new(max: i32) -> Self {
		Self {
			users: HashMap::new(),
			max_index: max,
			user_index: 0,
			master_index: -1,
		}
	}

	fn next_id(&mut self) -> Option<i32> {
		for _ in 0..self.max_index {
			self.user_index = (self.user_index % self.max_index) + 1;
			if !self.contains_key(&self.user_index) {
				return Some(self.user_index);
			}
		}

		None
	}

	pub fn add(&mut self, mut socket: TcpStream) -> bool {
		let Some(id) = self.next_id() else {
			return false;
		};

		let buf = [&b"hello\0"[..], &id.to_be_bytes(), &id.to_be_bytes()].concat();

		if let Ok(n) = socket.write(&buf) {
			if n != buf.len() {
				return false;
			}
		}

		let user = User::new(id, socket);
		self.insert(id, user);

		true
	}

	pub fn master(&mut self) -> Option<i32> {
		if self.users.get(&self.master_index).is_some() {
			return Some(self.master_index);
		}

		if let Some((_, user)) = self.users.iter_mut().next() {
			self.master_index = user.id;

			user.send(&ByteWriter::general_message(
				user.id,
				user.id,
				Opcode::SMsgSetMaster,
				&[1u8],
			));

			return Some(self.master_index);
		}

		None
	}

	pub fn for_others<F>(&mut self, id: &i32, f: F)
	where
		F: Fn(&mut User, &mut User),
	{
		let user_raw = self.users.get_mut(id).unwrap() as *mut User;
		for (_, other) in self.users.iter_mut() {
			if other as *mut User == user_raw {
				continue;
			}

			unsafe {
				f(&mut *user_raw, other);
			}
		}
	}

	pub fn send_user_count(&mut self) {
		let count = self.len();
		for (_, user) in self.iter_mut() {
			user.send(&ByteWriter::general_message(
				0,
				0,
				Opcode::SMsgUserCount,
				&ByteWriter::new(5).write_u8(1).write_i32(count as i32).bytes,
			))
		}
	}
}
