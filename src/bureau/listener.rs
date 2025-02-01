use std::{
	io::{self, ErrorKind, Read},
	net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
	time::Instant,
};

pub enum ListenerEvent {
	Incoming(SocketAddr),
	Accepted(TcpStream),
}

pub struct Listener {
	port: u16,
	timeout: u64,
	listener: TcpListener,
	queue: Vec<(TcpStream, Instant)>,
}

impl Listener {
	pub fn new<A: ToSocketAddrs>(addr: A, timeout: u64) -> io::Result<Self> {
		let listener = TcpListener::bind(addr)?;
		listener.set_nonblocking(true)?;

		let port = listener.local_addr()?.port();

		Ok(Self {
			port,
			timeout,
			listener,
			queue: Vec::new(),
		})
	}

	pub fn port(&self) -> u16 {
		self.port
	}

	pub fn deny_last(&mut self) {
		self.queue.pop();
	}

	pub fn poll_event(&mut self) -> io::Result<Option<ListenerEvent>> {
		match self.listener.accept() {
			Ok((stream, addr)) => {
				stream.set_nonblocking(true)?;
				self.queue.push((stream, Instant::now()));

				return Ok(Some(ListenerEvent::Incoming(addr)));
			}
			Err(e) if e.kind() == ErrorKind::WouldBlock => (),
			Err(e) => return Err(e),
		}

		let mut index = 0;
		'outer: while index < self.queue.len() {
			let (stream, connect_time) = &mut self.queue[index];

			if connect_time.elapsed().as_secs() > self.timeout {
				self.queue.remove(index);
				continue;
			}

			// 'hello' + 2 bytes
			let mut buf: [u8; 7] = [0; 7];
			let n = match stream.read(&mut buf) {
				Ok(n) => n,
				Err(e) if e.kind() == ErrorKind::WouldBlock => {
					index += 1; // Nothing was removed, advance index
					continue;
				}
				Err(e) => {
					self.queue.remove(index);

					return Err(e);
				}
			};

			let (stream, _) = self.queue.remove(index);

			if n < buf.len() {
				continue;
			}

			for (i, b) in buf.iter().enumerate() {
				// last two bytes are VSCP version (major then minor)
				if *b != b"hello\x01\x01"[i] {
					continue 'outer;
				}
			}

			return Ok(Some(ListenerEvent::Accepted(stream)));
		}

		Ok(None)
	}
}
