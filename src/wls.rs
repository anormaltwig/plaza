use std::{
	collections::{HashMap, HashSet},
	io::{self, Read, Write},
	net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
	thread,
	time::{Duration, SystemTime},
};

use crate::bureau::{Bureau, BureauHandle, BureauOptions};

pub struct WlsOptions {
	pub max_bureaus: u32,
	pub host_name: String,
	pub port: u16,
	pub bureau_options: BureauOptions,
}

pub struct Wls {
	options: WlsOptions,
	listener: TcpListener,
	bureaus: HashMap<String, HashMap<u16, BureauHandle>>,
}

impl Wls {
	pub fn start(options: WlsOptions) -> io::Result<()> {
		let listener = TcpListener::bind(SocketAddr::new(
			IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
			options.port,
		))?;
		listener.set_nonblocking(true)?;

		let mut bureaus = HashMap::new();
		for wrl in Self::default_wrls() {
			bureaus.insert(wrl, HashMap::new());
		}

		Self {
			options,
			listener,
			bureaus,
		}
		.run();

		Ok(())
	}

	fn default_wrls() -> HashSet<String> {
		let mut list = HashSet::new();

		list.insert("SAPARi COAST MIL.".to_string());
		list.insert("SAPARi DOWNTOWN MIL.".to_string());
		list.insert("HONJO JIDAIMURA MIL.".to_string());
		list.insert("SAPARi PARK MIL.".to_string());
		list.insert("SAPARi SPA".to_string());

		list
	}

	fn run(&mut self) {
		let mut connecting = Vec::new();
		loop {
			let now = SystemTime::now();

			if let Ok((socket, _addr)) = self.listener.accept() {
				if let Ok(()) = socket.set_nonblocking(true) {
					connecting.push((now.clone(), socket));
				}
			}

			let mut i = 0;
			while i < connecting.len() {
				let (connect_time, socket) = &mut connecting[i];

				let mut buf = [0; 128];
				let n = match socket.read(&mut buf) {
					Ok(n) => n,
					Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
						if let Ok(duration) = now.duration_since(*connect_time) {
							if duration.as_secs() > 10 {
								connecting.swap_remove(i);
							}
						} else {
							i += 1;
						}

						continue;
					}
					Err(_) => {
						connecting.swap_remove(i);
						continue;
					}
				};

				let mut socket = connecting.swap_remove(i).1;

				if n < 3 {
					continue;
				}

				let request = match String::from_utf8(buf[..n].to_vec()) {
					Ok(s) => s,
					Err(_) => continue,
				};

				let mut split = request.split(',');

				match split.next() {
					Some(f) if f == "f" => (),
					_ => continue,
				}

				// Local IP
				if let None = split.next() {
					continue;
				}

				// World Name
				let wrl = match split.next() {
					Some(wrl) => wrl,
					None => continue,
				};

				let wrl_bureaus = match self.bureaus.get_mut(wrl) {
					Some(b) => b,
					None => {
						let _ = socket.write_all(b"f,9");
						continue;
					}
				};

				if let Some((port, _)) = wrl_bureaus.iter().find(|(_, b)| !b.is_full()) {
					let _ = socket
						.write_all(format!("f,0,{},{}\0", self.options.host_name, port).as_bytes());
					continue;
				}

				if (wrl_bureaus.len() as u32) < self.options.max_bureaus {
					let bureau = match Bureau::new(
						SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
						self.options.bureau_options.clone(),
					) {
						Ok(b) => b,
						Err(io_err) => {
							eprintln!("Bureau failed to start. {io_err}");
							let _ = socket.write_all(b"f,9");
							continue;
						}
					};

					let _ = socket.write_all(
						format!("f,0,{},{}\0", self.options.host_name, bureau.port).as_bytes(),
					);

					wrl_bureaus.insert(bureau.port, bureau);
				}
			}

			for (_, bureaus) in &mut self.bureaus {
				for (_, _bureau) in bureaus {
					// Close empty bureaus and receive signals here.
				}
			}

			thread::sleep(Duration::from_millis(100))
		}
	}
}
