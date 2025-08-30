use std::{
	io::{self, Write},
	net::TcpStream,
	rc::Rc,
};

#[cfg(not(debug_assertions))]
use std::cell::UnsafeCell;

#[cfg(debug_assertions)]
use std::cell::{Ref, RefCell, RefMut};

use hashbrown::HashMap;

use super::{
	protocol::{ByteWriter, Opcode},
	user::User,
};

pub struct UserList {
	pub users: HashMap<i32, User>,

	master_id: i32,

	max_index: i32,
	user_index: i32,
}

impl UserList {
	pub fn new(max_users: i32) -> Self {
		Self {
			users: HashMap::new(),
			master_id: -1,

			max_index: max_users,
			user_index: 0,
		}
	}

	fn next_id(&mut self) -> Option<i32> {
		for _ in 0..self.max_index {
			self.user_index = (self.user_index % (self.max_index + 1)) + 1;
			if !self.users.contains_key(&self.user_index) {
				return Some(self.user_index);
			}
		}

		None
	}

	#[rustfmt::skip]
	const REJECT_BUF: [u8; 14] = [
		b'r', b'e', b'j', b'e', b'c', b't', 0,
		0, 0, 0, 0, 0, 0, 0,
	];

	pub fn add(&mut self, mut stream: TcpStream) -> io::Result<bool> {
		let Some(id) = self.next_id() else {
			let _ = stream.write(&Self::REJECT_BUF);
			return Ok(false);
		};

		let id_bytes = id.to_be_bytes();

		#[rustfmt::skip]
		let buf = [
			b'h', b'e', b'l', b'l', b'o', 69, // glorious unused 6th byte
			id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3],
			id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3],
		];

		if let Ok(n) = stream.write(&buf) {
			if n != buf.len() {
				return Ok(false);
			}
		}
		self.users.insert(id, User::new(id, stream)?);

		Ok(true)
	}

	pub fn master(&mut self) -> Option<i32> {
		if self.users.contains_key(&self.master_id) {
			return Some(self.master_id);
		}

		if let Some((_, user)) = self.users.iter_mut().next() {
			self.master_id = user.id();

			user.send(
				&ByteWriter::general_message(user.id(), user.id(), Opcode::SMsgSetMaster, &[1u8])
					.bytes,
			);

			return Some(self.master_id);
		}

		None
	}

	pub fn disconnect(&mut self, id: i32) {
		self.for_aura(id, |user, other| other.remove_aura(user));
		self.users.remove(&id);
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
		for other_id in user.aura().iter() {
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
		let count = self.users.len();
		let msg = ByteWriter::general_message(
			0,
			0,
			Opcode::SMsgUserCount,
			&ByteWriter::new(5).write_u8(1).write_i32(count as i32).bytes,
		);

		for user in self.users.values_mut() {
			user.send(&msg.bytes);
		}
	}

	pub fn send_all(&mut self, bytes: &[u8]) {
		for user in self.users.values_mut() {
			user.send(bytes);
		}
	}

	pub fn send_others(&mut self, id: i32, buf: &[u8]) {
		self.for_others(id, |_, other| {
			other.send(buf);
		});
	}

	pub fn send_aura(&mut self, id: i32, buf: &[u8]) {
		self.for_aura(id, |_, other| {
			other.send(buf);
		});
	}
}

/// Rc RefCell that gets turned into Rc UnsafeCell when compiled in release
pub struct AwesomeCell<T> {
	#[cfg(not(debug_assertions))]
	inner: Rc<UnsafeCell<T>>,

	#[cfg(debug_assertions)]
	inner: Rc<RefCell<T>>,
}

#[allow(dead_code)]
impl<T> AwesomeCell<T> {
	pub fn new(val: T) -> Self {
		Self {
			#[cfg(not(debug_assertions))]
			inner: Rc::new(UnsafeCell::new(val)),

			#[cfg(debug_assertions)]
			inner: Rc::new(RefCell::new(val)),
		}
	}

	#[cfg(not(debug_assertions))]
	pub fn get(&self) -> &T {
		unsafe { &*self.inner.get() }
	}

	#[cfg(debug_assertions)]
	pub fn get(&self) -> Ref<'_, T> {
		self.inner.borrow()
	}

	#[allow(clippy::mut_from_ref)]
	#[cfg(not(debug_assertions))]
	pub fn get_mut(&self) -> &mut T {
		unsafe { &mut *self.inner.get() }
	}

	#[cfg(debug_assertions)]
	pub fn get_mut(&self) -> RefMut<'_, T> {
		self.inner.borrow_mut()
	}
}

impl<T> Clone for AwesomeCell<T> {
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
		}
	}
}
