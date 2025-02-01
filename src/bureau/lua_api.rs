use std::{
	fs,
	io::{self, ErrorKind},
	net::{IpAddr, SocketAddr},
	path::PathBuf,
};

use mlua::{ChunkMode, FromLuaMulti, Function, IntoLuaMulti, Lua, RegistryKey, Table};

use super::{
	math::{Mat3, Vector3},
	protocol::{ByteWriter, MsgCommon, Strategy},
	user_list::{AwesomeCell, UserList},
};

struct Funcs {
	think: RegistryKey,
	user_connect: RegistryKey,
	new_user: RegistryKey,
	pos_update: RegistryKey,
	trans_update: RegistryKey,
	chat_send: RegistryKey,
	name_change: RegistryKey,
	avatar_change: RegistryKey,
	private_chat: RegistryKey,
	// aura_enter: RegistryKey,
	// aura_leave: RegistryKey,
	user_disconnect: RegistryKey,
	plugins_loaded: RegistryKey,
}

#[allow(unused_mut)]
impl Funcs {
	pub fn init(lua: &mut Lua, user_list: AwesomeCell<UserList>) -> mlua::Result<Self> {
		let tbl = lua.create_table()?;

		tbl.set(
			"set_pos",
			lua.create_function({
				let user_list = user_list.clone();
				move |_, (id, x, y, z): (i32, f32, f32, f32)| {
					let mut ul = user_list.get_mut();
					let Some(user) = ul.users.get_mut(&id) else {
						return Err(mlua::Error::external("invalid user"));
					};

					user.set_pos(Vector3::new(x, y, z));
					Ok(())
				}
			})?,
		)?;

		tbl.set(
			"set_rot",
			lua.create_function({
				let user_list = user_list.clone();
				move |_, (id, arr): (i32, [f32; 9])| {
					let mut ul = user_list.get_mut();
					let Some(user) = ul.users.get_mut(&id) else {
						return Err(mlua::Error::external("invalid user"));
					};

					user.set_rot(Mat3 { data: arr });
					Ok(())
				}
			})?,
		)?;

		tbl.set(
			"send_msg",
			lua.create_function({
				let user_list = user_list.clone();
				move |_, (id, msg): (i32, String)| {
					let mut ul = user_list.get_mut();
					let Some(user) = ul.users.get_mut(&id) else {
						return Err(mlua::Error::external("invalid user"));
					};

					user.send(
						&ByteWriter::message_common(
							user.id(),
							user.id(),
							MsgCommon::ChatSend,
							Strategy::AllClientsExceptSender,
							&ByteWriter::new(msg.len() + 1).write_string(&msg).bytes,
						)
						.bytes,
					);
					Ok(())
				}
			})?,
		)?;

		tbl.set(
			"send_packet",
			lua.create_function({
				let user_list = user_list.clone();
				move |_, (id, msg): (i32, mlua::String)| {
					let mut ul = user_list.get_mut();
					let Some(user) = ul.users.get_mut(&id) else {
						return Err(mlua::Error::external("invalid user"));
					};

					user.send(&msg.as_bytes());
					Ok(())
				}
			})?,
		)?;

		tbl.set(
			"disconnect",
			lua.create_function({
				let user_list = user_list.clone();
				move |_, id: i32| {
					let mut ul = user_list.get_mut();
					let Some(user) = ul.users.get_mut(&id) else {
						return Err(mlua::Error::external("invalid user"));
					};

					user.disconnect();
					Ok(())
				}
			})?,
		)?;

		lua.load(include_str!("lua/vector.lua")).exec()?;
		lua.load(include_str!("lua/basis.lua")).exec()?;

		let (users, user_meta): (Table, Table) =
			lua.load(include_str!("lua/user.lua")).call(tbl)?;

		let tbl: Table = lua
			.load(include_str!("lua/hook.lua"))
			.call((users, user_meta))?;

		Ok(Self {
			think: lua.create_registry_value(tbl.get::<Function>("think")?)?,
			user_connect: lua.create_registry_value(tbl.get::<Function>("user_connect")?)?,
			new_user: lua.create_registry_value(tbl.get::<Function>("new_user")?)?,
			pos_update: lua.create_registry_value(tbl.get::<Function>("pos_update")?)?,
			trans_update: lua.create_registry_value(tbl.get::<Function>("trans_update")?)?,
			chat_send: lua.create_registry_value(tbl.get::<Function>("chat_send")?)?,
			name_change: lua.create_registry_value(tbl.get::<Function>("name_change")?)?,
			avatar_change: lua.create_registry_value(tbl.get::<Function>("avatar_change")?)?,
			private_chat: lua.create_registry_value(tbl.get::<Function>("private_chat")?)?,
			// aura_enter: lua.create_registry_value(tbl.get::<Function>("aura_enter")?)?,
			// aura_leave: lua.create_registry_value(tbl.get::<Function>("aura_leave")?)?,
			user_disconnect: lua.create_registry_value(tbl.get::<Function>("user_disconnect")?)?,
			plugins_loaded: lua.create_registry_value(tbl.get::<Function>("plugins_loaded")?)?,
		})
	}
}

pub struct LuaApi {
	lua: Lua,
	funcs: Funcs,
}

fn do_file(lua: &mut Lua, path: PathBuf) -> mlua::Result<()> {
	let chunkname = format!("={:?}", path);

	let data = fs::read(path)?;
	lua.load(data)
		.set_mode(ChunkMode::Text)
		.set_name(chunkname)
		.exec()?;

	Ok(())
}

fn load_plugins(lua: &mut Lua) -> io::Result<()> {
	let read_dir = match fs::read_dir("plugins") {
		Ok(r) => r,
		Err(err) => {
			if err.kind() == ErrorKind::NotFound {
				println!("The 'plugins' directory is missing, no plugins will be loaded.");
				return Ok(());
			}

			return Err(err);
		}
	};

	for file in read_dir {
		let file = file?;

		if file.file_type()?.is_dir() {
			let path = file.path();
			let initpath = path.join("init.lua");

			if !initpath.is_file() {
				eprintln!("{:?} is missing an init.lua and will not be loaded.", path);
				continue;
			}

			if let Err(e) = do_file(lua, initpath) {
				eprintln!("Error while loading plugin {:?}, {}", path, e)
			}
		}
	}

	Ok(())
}

impl LuaApi {
	pub fn new(user_list: AwesomeCell<UserList>) -> mlua::Result<Self> {
		let mut lua = unsafe { Lua::unsafe_new() };

		let funcs = Funcs::init(&mut lua, user_list)?;
		load_plugins(&mut lua)?;

		let lua_api = Self { lua, funcs };

		lua_api.call::<_, ()>(&lua_api.funcs.plugins_loaded, ());

		Ok(lua_api)
	}

	fn call<A, R>(&self, rk: &RegistryKey, args: A) -> Option<R>
	where
		A: IntoLuaMulti,
		R: FromLuaMulti,
	{
		let f = self.lua.registry_value::<Function>(rk).ok()?;
		match f.call::<R>(args) {
			Ok(r) => Some(r),
			Err(e) => {
				eprintln!("Lua Error: {}", e);
				None
			}
		}
	}

	pub fn think(&mut self) {
		let _ = self.call::<_, ()>(&self.funcs.think, ());
	}

	pub fn user_connect(&self, addr: SocketAddr) -> bool {
		self.call::<_, Option<bool>>(&self.funcs.user_connect, addr.ip().to_string())
			.flatten()
			.unwrap_or(true)
	}

	pub fn new_user(&self, id: i32, name: &str, avatar: &str, ip: IpAddr) {
		self.call::<_, ()>(&self.funcs.new_user, (id, name, avatar, ip.to_string()));
	}

	pub fn pos_update(&self, id: i32, pos: &Vector3) {
		let _ = self.call::<_, ()>(&self.funcs.pos_update, (id, pos.x, pos.y, pos.z));
	}

	pub fn trans_update(&self, id: i32, rot: &Mat3) {
		let _ = self.call::<_, ()>(&self.funcs.trans_update, (id, rot.data));
	}

	pub fn chat_send(&self, id: i32, msg: &str) -> Option<String> {
		self.call::<_, Option<String>>(&self.funcs.chat_send, (id, msg))?
	}

	pub fn name_change(&self, id: i32, name: &str) {
		let _ = self.call::<_, Option<String>>(&self.funcs.name_change, (id, name));
	}

	pub fn avatar_change(&self, id: i32, avatar: &str) {
		let _ = self.call::<_, Option<String>>(&self.funcs.avatar_change, (id, avatar));
	}

	pub fn private_chat(&self, id1: i32, id2: i32, msg: &str) -> Option<String> {
		self.call::<_, Option<String>>(&self.funcs.private_chat, (id1, id2, msg))?
	}

	/*
	pub fn aura_enter(&self, id1: i32, id2: i32) {
		let _ = self.call::<_, Option<String>>(&self.funcs.aura_enter, (id1, id2));
	}

	pub fn aura_leave(&self, id1: i32, id2: i32) {
		let _ = self.call::<_, Option<String>>(&self.funcs.aura_leave, (id1, id2));
	}
	*/

	pub fn user_disconnect(&self, id: i32) {
		let _ = self.call::<_, Option<String>>(&self.funcs.user_disconnect, id);
	}
}
