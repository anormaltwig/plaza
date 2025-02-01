pub mod bureau_manager;

use std::{
	collections::HashMap,
	fs::File,
	io::{self, BufRead, BufReader, ErrorKind, Read, Write},
	net::{SocketAddrV4, TcpListener},
	thread,
	time::{Duration, Instant},
};

use bureau_manager::BureauManager;

use crate::bureau::BureauConfig;

pub struct WlsOptions {
	pub host_name: String,
	pub max_bureaus: usize,
	pub wrl_list: Option<String>,
	pub bureau_config: BureauConfig,
}

fn default_wrls() -> Vec<String> {
	vec![
		"SAPARi COAST MIL.".into(),
		"SAPARi DOWNTOWN MIL.".into(),
		"HONJO JIDAIMURA MIL.".into(),
		"SAPARi PARK MIL.".into(),
		"SAPARi SPA".into(),
		"SAPARi GARDEN".into(),
		"SAPARi HILLS".into(),
	]
}

pub fn run(addr: SocketAddrV4, options: WlsOptions) -> io::Result<()> {
	let listener = TcpListener::bind(addr)?;
	listener.set_nonblocking(true)?;
	let wls_port = listener.local_addr()?.port();

	let wrls = match &options.wrl_list {
		Some(path) => {
			let reader = BufReader::new(File::open(path)?);
			reader.lines().map_while(Result::ok).collect()
		}
		None => default_wrls(),
	};

	let mut managers = HashMap::with_capacity(wrls.len());
	for wrl in wrls {
		managers.insert(
			wrl,
			BureauManager::new(options.max_bureaus, options.bureau_config.clone()),
		);
	}

	let mut queue = Vec::new();

	println!("WLS running on port: {}.", wls_port);
	loop {
		if let Ok((socket, _)) = listener.accept() {
			if let Ok(()) = socket.set_nonblocking(true) {
				queue.push((Instant::now(), socket));
			}
		}

		queue.retain_mut(|(connect_time, socket)| {
			let mut buf = [0; 256];
			let n = match socket.read(&mut buf) {
				Ok(n) => n,
				Err(e) if e.kind() == ErrorKind::WouldBlock => {
					return connect_time.elapsed().as_secs() < 10;
				}
				Err(_) => return false,
			};

			let Ok(request) = String::from_utf8(buf[..n].to_vec()) else {
				return false;
			};

			let mut split = request.split(',');

			let Some("f") = split.next() else {
				return false;
			};

			if split.next().is_none() {
				return false;
			}

			let Some(wrl) = split.next() else {
				return false;
			};

			let Some(port) = (match managers.get_mut(wrl) {
				Some(manager) => manager.available(),
				None => None,
			}) else {
				let _ = socket.write(b"f,9");
				return false;
			};

			let _ = socket.write(format!("f,0,{},{}\0", options.host_name, port).as_bytes());

			false
		});

		for manager in managers.values_mut() {
			manager.poll();
		}

		thread::sleep(Duration::from_millis(100));
	}
}
