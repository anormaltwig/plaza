use std::{
	cell::RefCell,
	fs,
	io::{self, ErrorKind},
	net::{IpAddr, SocketAddr},
	path::PathBuf,
	rc::Rc,
};

use branch_macro::include_lua;
use mlua::{ChunkMode, FromLuaMulti, Function, IntoLuaMulti, Lua, RegistryKey, Table};

use super::{
	math::{Mat3, Vector3},
	protocol::{ByteWriter, MsgCommon, Strategy},
	user_list::UserList,
};

type EventQueue = Rc<RefCell<Vec<(i32, LuaEvent)>>>;

pub enum LuaEvent {
	SetPos(Vector3),
	SetRot(Mat3),
	SendMsg(String),
	SendPacket(ByteWriter),
	Disconnect,
}

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
	aura_enter: RegistryKey,
	aura_leave: RegistryKey,
	user_disconnect: RegistryKey,
	plugins_loaded: RegistryKey,
}

impl Funcs {
	pub fn init(lua: &mut Lua, event_queue: &EventQueue) -> mlua::Result<Self> {
		let tbl = lua.create_table()?;

		tbl.set(
			"set_pos",
			lua.create_function({
				let event_queue = event_queue.clone();
				move |_, (id, x, y, z): (i32, f32, f32, f32)| {
					event_queue
						.borrow_mut()
						.push((id, LuaEvent::SetPos(Vector3::new(x, y, z))));
					Ok(())
				}
			})?,
		)?;

		tbl.set(
			"set_rot",
			lua.create_function({
				let event_queue = event_queue.clone();
				move |_, (id, arr): (i32, [f32; 9])| {
					event_queue
						.borrow_mut()
						.push((id, LuaEvent::SetRot(Mat3 { data: arr })));
					Ok(())
				}
			})?,
		)?;

		tbl.set(
			"send_msg",
			lua.create_function({
				let event_queue = event_queue.clone();
				move |_, (id, msg): (i32, String)| {
					event_queue.borrow_mut().push((id, LuaEvent::SendMsg(msg)));
					Ok(())
				}
			})?,
		)?;

		tbl.set(
			"send_packet",
			lua.create_function({
				let event_queue = event_queue.clone();
				move |_, (id, msg): (i32, mlua::String)| {
					event_queue.borrow_mut().push((
						id,
						LuaEvent::SendPacket(ByteWriter {
							bytes: msg.as_bytes().to_vec(),
						}),
					));
					Ok(())
				}
			})?,
		)?;

		tbl.set(
			"disconnect",
			lua.create_function({
				let event_queue = event_queue.clone();
				move |_, id: i32| {
					event_queue.borrow_mut().push((id, LuaEvent::Disconnect));
					Ok(())
				}
			})?,
		)?;

		lua.load(include_lua!("src/bureau/lua/vector.lua").as_ref())
			.exec()?;
		lua.load(include_lua!("src/bureau/lua/basis.lua").as_ref())
			.exec()?;

		let (users, user_meta): (Table, Table) = lua
			.load(include_lua!("src/bureau/lua/user.lua").as_ref())
			.call(tbl)?;

		let tbl: Table = lua
			.load(include_lua!("src/bureau/lua/hook.lua").as_ref())
			.call((users, user_meta))?;

		Ok(Self {
			think: lua.create_registry_value::<Function>(tbl.get("think")?)?,
			user_connect: lua.create_registry_value::<Function>(tbl.get("user_connect")?)?,
			new_user: lua.create_registry_value::<Function>(tbl.get("new_user")?)?,
			pos_update: lua.create_registry_value::<Function>(tbl.get("pos_update")?)?,
			trans_update: lua.create_registry_value::<Function>(tbl.get("trans_update")?)?,
			chat_send: lua.create_registry_value::<Function>(tbl.get("chat_send")?)?,
			name_change: lua.create_registry_value::<Function>(tbl.get("name_change")?)?,
			avatar_change: lua.create_registry_value::<Function>(tbl.get("avatar_change")?)?,
			private_chat: lua.create_registry_value::<Function>(tbl.get("private_chat")?)?,
			aura_enter: lua.create_registry_value::<Function>(tbl.get("aura_enter")?)?,
			aura_leave: lua.create_registry_value::<Function>(tbl.get("aura_leave")?)?,
			user_disconnect: lua.create_registry_value::<Function>(tbl.get("user_disconnect")?)?,
			plugins_loaded: lua.create_registry_value::<Function>(tbl.get("plugins_loaded")?)?,
		})
	}
}

pub struct LuaApi {
	lua: Lua,
	funcs: Funcs,
	event_queue: EventQueue,
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
	pub fn new() -> anyhow::Result<Self> {
		let mut lua = unsafe { Lua::unsafe_new() };

		let event_queue = Rc::new(RefCell::new(Vec::new()));
		let funcs = Funcs::init(&mut lua, &event_queue)?;
		load_plugins(&mut lua)?;

		let lua_api = Self {
			lua,
			funcs,
			event_queue,
		};

		lua_api.call::<_, ()>(&lua_api.funcs.plugins_loaded, ());

		Ok(lua_api)
	}

	pub fn run_events(&mut self, user_list: &mut UserList) {
		let mut event_queue = self.event_queue.borrow_mut();
		if event_queue.is_empty() {
			return;
		}

		for (id, event) in event_queue.drain(..) {
			let Some(user) = user_list.get_mut(&id) else {
				continue;
			};

			match event {
				LuaEvent::SetPos(pos) => user.set_pos(pos),
				LuaEvent::SetRot(rot) => user.set_rot(rot),
				LuaEvent::SendMsg(msg) => user.send(&ByteWriter::message_common(
					user.id,
					user.id,
					MsgCommon::ChatSend,
					Strategy::AllClientsExceptSender,
					&ByteWriter::new(msg.len() + 1).write_string(&msg).bytes,
				)),
				LuaEvent::SendPacket(packet) => user.send(&packet),
				LuaEvent::Disconnect => user.connected = false,
			}
		}

		event_queue.shrink_to_fit();
	}

	fn call<A, R>(&self, rk: &RegistryKey, args: A) -> Option<R>
	where
		A: for<'a> IntoLuaMulti<'a>,
		R: for<'a> FromLuaMulti<'a>,
	{
		let f = self.lua.registry_value::<Function>(rk).ok()?;
		match f.call::<A, R>(args) {
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

	pub fn aura_enter(&self, id1: i32, id2: i32) {
		let _ = self.call::<_, Option<String>>(&self.funcs.aura_enter, (id1, id2));
	}

	pub fn aura_leave(&self, id1: i32, id2: i32) {
		let _ = self.call::<_, Option<String>>(&self.funcs.aura_leave, (id1, id2));
	}

	pub fn user_disconnect(&self, id: i32) {
		let _ = self.call::<_, Option<String>>(&self.funcs.user_disconnect, id);
	}
}
