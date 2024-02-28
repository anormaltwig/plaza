use std::{
	cell::RefCell,
	collections::HashSet,
	io::{self, Read, Write},
	net::TcpStream,
};

use crate::core::math::{Mat3, Vector3};

use super::protocol::ByteStream;

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
	pub fn new(socket: TcpStream, id: i32) -> io::Result<User> {
		socket.set_nonblocking(true)?;
		Ok(User {
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

	pub fn read(&self, buf: &mut [u8]) -> Option<usize> {
		let mut internal = self.internal.borrow_mut();

		match internal.socket.read(buf) {
			Ok(n) => {
				if n == 0 {
					internal.connected = false;

					None
				} else {
					Some(n)
				}
			}
			Err(_) => None,
		}
	}

	pub fn send(&self, stream: &ByteStream) {
		let mut internal = self.internal.borrow_mut();

		if let Err(_) = internal.socket.write_all(&stream.bytes) {
			internal.connected = false;
		}
	}

	pub fn is_connected(&self) -> bool {
		self.internal.borrow().connected
	}

	pub fn set_pos(&self, pos: Vector3) {
		self.internal.borrow_mut().position = pos;
	}
	pub fn get_pos(&self) -> Vector3 {
		self.internal.borrow().position.clone()
	}

	pub fn set_rot(&self, rot: Mat3) {
		self.internal.borrow_mut().rotation = rot;
	}
	pub fn get_rot(&self) -> Mat3 {
		self.internal.borrow().rotation.clone()
	}

	pub fn set_name(&self, name: String) {
		self.internal.borrow_mut().username = name
	}
	pub fn get_name(&self) -> String {
		self.internal.borrow().username.clone()
	}

	pub fn set_avatar(&self, avatar: String) {
		self.internal.borrow_mut().avatar = avatar
	}
	pub fn get_avatar(&self) -> String {
		self.internal.borrow().avatar.clone()
	}

	pub fn set_data(&self, data: String) {
		self.internal.borrow_mut().data = data;
	}
	pub fn get_data(&self) -> String {
		self.internal.borrow().data.clone()
	}

	pub fn add_aura(&self, id: i32) -> bool {
		self.internal.borrow_mut().aura.insert(id)
	}
	pub fn check_aura(&self, id: &i32) -> bool {
		self.internal.borrow().aura.contains(id)
	}
	pub fn remove_aura(&self, id: &i32) -> bool {
		self.internal.borrow_mut().aura.remove(id)
	}
	pub fn get_aura(&self) -> HashSet<i32> {
		self.internal.borrow_mut().aura.clone()
	}
}
