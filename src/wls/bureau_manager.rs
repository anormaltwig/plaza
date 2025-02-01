use std::{
	net::{Ipv4Addr, SocketAddrV4},
	time::Instant,
};

use crate::bureau::{Bureau, BureauConfig};

struct BureauEx {
	start_time: Instant,
	inner: Bureau,
}

pub struct BureauManager {
	bureaus: Vec<BureauEx>,
	max: usize,
	bureau_options: BureauConfig,
}

impl BureauManager {
	const BIND_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);

	pub fn new(max: usize, bureau_options: BureauConfig) -> Self {
		Self {
			bureaus: Vec::with_capacity(max),
			max,
			bureau_options,
		}
	}

	pub fn poll(&mut self) {
		self.bureaus.retain_mut(|bureau_ex| {
			if let Err(err) = bureau_ex.inner.poll() {
				eprintln!("error during bureau loop {}", err);
			}

			bureau_ex.start_time.elapsed().as_secs() < 10 || bureau_ex.inner.user_count() > 0
		})
	}

	pub fn available(&mut self) -> Option<u16> {
		if let Some(bureau_ex) = self.bureaus.iter().find(|bureau_ex| {
			bureau_ex.inner.user_count() < bureau_ex.inner.config().max_users as usize
		}) {
			return Some(bureau_ex.inner.port());
		}

		if self.bureaus.len() < self.max {
			let bureau = Bureau::new(Self::BIND_ADDR, self.bureau_options.clone()).ok()?;
			let port = bureau.port();

			self.bureaus.push(BureauEx {
				start_time: Instant::now(),
				inner: bureau,
			});

			return Some(port);
		}

		None
	}
}
