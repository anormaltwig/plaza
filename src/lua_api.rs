use std::{cell::RefCell, collections::HashMap, fs, path::PathBuf, rc::Rc};

use anyhow::Result;
use mlua::{ChunkMode, Error, Function, Lua, RegistryKey, Table};

use crate::{
	math::{Mat3, Vector3},
	user::User,
};

pub struct LuaApi {
	lua: Lua,
	think: RegistryKey,
	new_user: RegistryKey,
	pos_update: RegistryKey,
	trans_update: RegistryKey,
	chat_send: RegistryKey,
	name_change: RegistryKey,
	avatar_change: RegistryKey,
}

impl LuaApi {
	pub fn new(users: Rc<RefCell<HashMap<i32, User>>>) -> Result<Self> {
		let lua = Lua::new();

		let data = fs::read("lua/init.lua")?;

		let fn_tbl: Table = lua
			.load(data)
			.set_mode(ChunkMode::Text)
			.set_name("=branch")
			.call(Self::create_funcs(&lua, users)?)?;

		let think = lua.create_registry_value(fn_tbl.get::<_, Function>("think")?)?;
		let new_user = lua.create_registry_value(fn_tbl.get::<_, Function>("new_user")?)?;
		let pos_update = lua.create_registry_value(fn_tbl.get::<_, Function>("pos_update")?)?;
		let trans_update = lua.create_registry_value(fn_tbl.get::<_, Function>("trans_update")?)?;
		let chat_send = lua.create_registry_value(fn_tbl.get::<_, Function>("chat_send")?)?;
		let name_change = lua.create_registry_value(fn_tbl.get::<_, Function>("name_change")?)?;
		let avatar_change = lua.create_registry_value(fn_tbl.get::<_, Function>("avatar_change")?)?;

		drop(fn_tbl);

		let this = Self {
			lua,
			think,
			new_user,
			pos_update,
			trans_update,
			chat_send,
			name_change,
			avatar_change,
		};

		this.load_plugins()?;

		Ok(this)
	}

	fn create_funcs(lua: &Lua, users: Rc<RefCell<HashMap<i32, User>>>) -> Result<Table> {
		let tbl = lua.create_table()?;

		let u = users.clone();
		tbl.set(
			"set_pos",
			lua.create_function(move |_lua: &Lua, (id, x, y, z): (i32, f32, f32, f32)| {
				let users = u.borrow();
				let user = users.get(&id).ok_or(Error::RuntimeError(
					"Tried to use invalid User.".to_string(),
				))?;

				user.set_pos(&Vector3::new(x, y, z));

				Ok(())
			})?,
		)?;

		let u = users.clone();
		tbl.set(
			"get_pos",
			lua.create_function(move |_lua: &Lua, id: i32| {
				let users = u.borrow();
				let user = users.get(&id).ok_or(Error::RuntimeError(
					"Tried to use invalid User.".to_string(),
				))?;

				let pos = user.get_pos();

				Ok((pos.x, pos.y, pos.z))
			})?,
		)?;

		let u = users.clone();
		tbl.set(
			"set_rot",
			lua.create_function(move |_lua: &Lua, (id, arr): (i32, [f32; 9])| {
				let users = u.borrow();
				let user = users.get(&id).ok_or(Error::RuntimeError(
					"Tried to use invalid User.".to_string(),
				))?;

				let mut m = Mat3::new();
				m.data = arr;
				user.set_rot(m);

				Ok(())
			})?,
		)?;

		let u = users.clone();
		tbl.set(
			"get_rot",
			lua.create_function(move |_lua: &Lua, id: i32| {
				let users = u.borrow();
				let user = users.get(&id).ok_or(Error::RuntimeError(
					"Tried to use invalid User.".to_string(),
				))?;

				Ok(user.get_rot().data)
			})?,
		)?;

		let u = users.clone();
		tbl.set(
			"send_msg",
			lua.create_function(move |_lua: &Lua, (id, msg): (i32, String)| {
				let users = u.borrow();
				let user = users.get(&id).ok_or(Error::RuntimeError(
					"Tried to use invalid User.".to_string(),
				))?;

				user.send_msg(&msg);

				Ok(())
			})?,
		)?;

		Ok(tbl)
	}

	fn do_file(&self, path: PathBuf) -> Result<()> {
		let chunkname = format!("={:?}", path);

		let data = fs::read(path)?;
		self.lua
			.load(data)
			.set_mode(ChunkMode::Text)
			.set_name(chunkname)
			.exec()?;

		Ok(())
	}

	fn load_plugins(&self) -> Result<()> {
		for f in fs::read_dir("plugins")? {
			let file = f?;
			if file.file_type()?.is_dir() {
				let path = file.path();
				let initpath = path.join("init.lua");

				if !initpath.is_file() {
					eprintln!("{:?} is missing an init.lua and will not be loaded.", path);
					continue;
				}

				if let Err(e) = self.do_file(initpath) {
					eprintln!("Error while loading plugin {:?}, {}", path, e)
				}
			}
		}

		Ok(())
	}

	pub fn think(&self) {
		let think = match self.lua.registry_value::<Function>(&self.think) {
			Ok(f) => f,
			Err(_) => return,
		};

		if let Err(e) = think.call::<_, ()>(()) {
			eprintln!("Lua Error:\n{}", e);
		}
	}

	pub fn new_user(&self, user: &User, name: &str, avatar: &str) {
		let new_user = match self.lua.registry_value::<Function>(&self.new_user) {
			Ok(f) => f,
			Err(_) => return,
		};

		if let Err(e) = new_user.call::<_, ()>((user.id, name, avatar)) {
			eprintln!("Lua Error:\n{}", e);
		}
	}

	pub fn pos_update(&self, user: &User, pos: &Vector3) {
		let pos_update = match self.lua.registry_value::<Function>(&self.pos_update) {
			Ok(f) => f,
			Err(_) => return,
		};

		if let Err(e) = pos_update.call::<_, ()>((user.id, pos.x, pos.y, pos.z)) {
			eprintln!("Lua Error:\n{}", e);
		}
	}

	pub fn trans_update(&self, user: &User) {
		let trans_update = match self.lua.registry_value::<Function>(&self.trans_update) {
			Ok(f) => f,
			Err(_) => return,
		};

		if let Err(e) = trans_update.call::<_, ()>(user.id) {
			eprintln!("Lua Error:\n{}", e);
		}
	}

	pub fn chat_send(&self, user: &User, msg: &str) -> Option<String> {
		let chat_send = self.lua.registry_value::<Function>(&self.chat_send).ok()?;

		match chat_send.call::<_, Option<String>>((user.id, msg)) {
			Ok(r) => r,
			Err(e) => {
				eprintln!("Lua Error:\n{}", e);
				None
			}
		}
	}

	pub fn name_change(&self, user: &User, name: &str) {
		let name_change = match self.lua.registry_value::<Function>(&self.name_change) {
			Ok(f) => f,
			Err(_) => return,
		};

		if let Err(e) = name_change.call::<_, ()>((user.id, name)) {
			eprintln!("Lua Error:\n{}", e);
		}
	}

	pub fn avatar_change(&self, user: &User, avatar: &str) {
		let avatar_change = match self.lua.registry_value::<Function>(&self.avatar_change) {
			Ok(f) => f,
			Err(_) => return,
		};

		if let Err(e) = avatar_change.call::<_, ()>((user.id, avatar)) {
			eprintln!("Lua Error:\n{}", e);
		}
	}
}
