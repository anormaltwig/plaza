use std::{
	collections::HashMap,
	io::{self, Read, Write},
	net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
	thread,
	time::{Duration, SystemTime},
};

use crate::bureau::{Bureau, BureauOptions};

pub struct WlsOptions {
	pub max_bureaus: u32,
	pub host_name: String,
	pub port: u16,
	pub bureau_options: BureauOptions,
}

pub struct Wls {
	options: WlsOptions,
	listener: TcpListener,
	bureaus: HashMap<String, Vec<Bureau>>,
}

impl Wls {
	pub fn start(options: WlsOptions) -> io::Result<()> {
		let listener = TcpListener::bind(SocketAddr::new(
			IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
			options.port,
		))?;
		listener.set_nonblocking(true)?;

		Self {
			options,
			listener,
			bureaus: HashMap::new(),
		}
		.run();

		Ok(())
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

				if let Some(f) = split.next() {
					if f != "f" {
						continue;
					}
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

				let _ = match self.bureaus.get(wrl) {
					Some(_b) => socket.write_all(format!("f,0,{},5126\0", self.options.host_name).as_bytes()),
					_ => socket.write_all(b"f,9"),
				};
			}

			thread::sleep(Duration::from_millis(100))
		}
	}
}
