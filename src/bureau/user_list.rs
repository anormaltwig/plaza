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

	/// Attempt to create a User from a TcpStream and add it to the list.
	/// Returns false if a User cannot be created.
	pub fn add(&mut self, mut socket: TcpStream) -> bool {
		let Some(id) = self.next_id() else {
			return false;
		};

		let buf = [b"hello\0".as_ref(), &id.to_be_bytes(), &id.to_be_bytes()].concat();

		if let Ok(n) = socket.write(&buf) {
			if n != buf.len() {
				return false;
			}
		}

		let Ok(user) = User::new(id, socket) else {
			return false;
		};
		self.insert(id, user);

		true
	}

	/// Get the id of the User currently assigned the role of master.
	pub fn master(&mut self) -> Option<i32> {
		if self.users.contains_key(&self.master_index) {
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

	/// Iterate over all Users in the UserList while keeping a mutable reference to `id`.
	pub fn for_others<F>(&mut self, id: i32, f: F)
	where
		F: Fn(&mut User, &mut User),
	{
		let mut user = self.users.remove(&id).unwrap();
		for (_, other) in self.users.iter_mut() {
			f(&mut user, other);
		}
		self.users.insert(id, user);
	}

	/// Iterate over all Users in the aura of `id` while keeping an immutable reference to `id`.
	pub fn for_aura<F>(&mut self, id: i32, f: F)
	where
		F: Fn(&User, &mut User),
	{
		let user = self.users.remove(&id).unwrap();
		for other_id in user.aura.iter() {
			let Some(other) = self.users.get_mut(other_id) else {
				eprintln!("Aura desync, {} has id {}.", id, other_id);
				continue;
			};

			f(&user, other);
		}
		self.users.insert(id, user);
	}

	/// Broadcast the current number of connected Users to all Users.
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
