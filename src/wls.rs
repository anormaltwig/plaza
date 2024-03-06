use std::{
	collections::HashMap,
	io::{self, Read, Write},
	net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
	thread,
	time::{Duration, SystemTime},
};

use crate::bureau::{Bureau, BureauOptions};

#[allow(dead_code)]
pub struct WLS {
	bureaus: HashMap<String, Bureau>,
}

// !!!Not Finished!!!

#[allow(dead_code)]
impl WLS {
	pub fn start(port: u16) -> io::Result<()> {
		let listener =
			TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port))?;
		listener.set_nonblocking(true)?;

		let mut connecting = Vec::new();
		loop {
			let now = SystemTime::now();

			if let Ok((socket, _addr)) = listener.accept() {
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
				let _wrl = match split.next() {
					Some(wrl) => wrl,
					None => continue,
				};

				let _bureau = Bureau::new(
					"0.0.0.0:5126",
					BureauOptions {
						max_players: 1,
						aura_radius: 100.0,
					},
				);

				socket.write_all(b"f,0,127.0.0.1,5126\0").unwrap();
			}

			thread::sleep(Duration::from_millis(100))
		}
	}
}
