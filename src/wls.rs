use std::{
	collections::HashMap,
	fs::File,
	io::{self, BufRead, BufReader, Read, Write},
	net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
	thread,
	time::{Duration, Instant},
};

use crate::bureau::{Bureau, BureauHandle, BureauOptions};

pub struct WlsOptions {
	pub host_name: String,
	pub max_bureaus: u32,
	pub wrl_list: Option<String>,
	pub port: u16,
	pub bureau_options: BureauOptions,
}

pub struct Wls {
	options: WlsOptions,
	listener: TcpListener,
	bureaus: HashMap<String, Vec<(u16, BureauHandle)>>,
}

impl Wls {
	pub fn start(options: WlsOptions) -> io::Result<()> {
		let listener = TcpListener::bind(SocketAddr::new(
			IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
			options.port,
		))?;
		listener.set_nonblocking(true)?;

		let mut bureaus = HashMap::new();

		let wrls = match &options.wrl_list {
			Some(path) => {
				let reader = BufReader::new(File::open(path)?);
				reader.lines().filter_map(|line| Some(line.ok()?)).collect()
			}
			None => Self::default_wrls(),
		};

		for wrl in wrls {
			bureaus.insert(wrl, Vec::new());
		}

		Self {
			options,
			listener,
			bureaus,
		}
		.run();

		Ok(())
	}

	fn default_wrls() -> Vec<String> {
		vec![
			"SAPARi COAST MIL.".to_string(),
			"SAPARi DOWNTOWN MIL.".to_string(),
			"HONJO JIDAIMURA MIL.".to_string(),
			"SAPARi PARK MIL.".to_string(),
			"SAPARi SPA".to_string(),
			"SAPARi GARDEN".to_string(),
			"SAPARi HILLS".to_string(),
		]
	}

	fn run(&mut self) {
		let mut connecting = Vec::new();
		loop {
			if let Ok((socket, _addr)) = self.listener.accept() {
				if let Ok(()) = socket.set_nonblocking(true) {
					connecting.push((Instant::now(), Some(socket)));
				}
			}

			connecting.retain_mut(|(connect_time, socket)| {
				let mut buf = [0; 128];
				let n = match socket.as_ref().unwrap().read(&mut buf) {
					Ok(n) => n,
					Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
						if connect_time.elapsed().as_secs() < 10 {
							return true;
						}
						return false;
					}
					Err(_) => return false,
				};

				let mut socket = socket.take().unwrap();
				if n < 3 {
					return false;
				}

				let request = match String::from_utf8(buf[..n].to_vec()) {
					Ok(s) => s,
					Err(_) => return false,
				};

				let mut split = request.split(',');

				match split.next() {
					Some(f) if f == "f" => (),
					_ => return false,
				}

				// Local IP
				if let None = split.next() {
					return false;
				}

				// World Name
				let wrl = match split.next() {
					Some(wrl) => wrl,
					None => return false,
				};

				let wrl_bureaus = match self.bureaus.get_mut(wrl) {
					Some(b) => b,
					None => {
						let _ = socket.write_all(b"f,9");
						return false;
					}
				};

				if let Some((port, _)) = wrl_bureaus
					.iter()
					.find(|(_, b)| b.user_count() < b.options.max_players)
				{
					let _ = socket
						.write_all(format!("f,0,{},{}\0", self.options.host_name, port).as_bytes());
					return false;
				}

				if (wrl_bureaus.len() as u32) < self.options.max_bureaus {
					let bureau = match Bureau::spawn(
						SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
						self.options.bureau_options.clone(),
					) {
						Ok(b) => b,
						Err(io_err) => {
							eprintln!("Bureau failed to start. {io_err}");
							let _ = socket.write_all(b"f,9");
							return false;
						}
					};

					let _ = socket.write_all(
						format!("f,0,{},{}\0", self.options.host_name, bureau.port).as_bytes(),
					);

					wrl_bureaus.push((bureau.port, bureau));
				}
				false
			});

			for (_, bureaus) in &mut self.bureaus {
				bureaus.retain_mut(|(_, bureau)| {
					if bureau.startup_time.elapsed().as_secs() > 10 && bureau.user_count() == 0
					{
						bureau.close();
						if let Err(thread_err) = bureau.join() {
							eprintln!("Bureau panicked! ({:?})", thread_err);
						}

						return false;
					}

					true
				});
			}

			thread::sleep(Duration::from_millis(100))
		}
	}
}
