use std::{
	net::{IpAddr, Ipv4Addr, SocketAddr},
	time::Instant,
};

use crate::bureau::{Bureau, BureauOptions};

struct BureauEx {
	start_time: Instant,
	inner: Bureau,
}

pub struct BureauManager {
	bureaus: Vec<BureauEx>,
	max: usize,
	bureau_options: BureauOptions,
}

impl BureauManager {
	const BIND_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

	pub fn new(max: usize, bureau_options: BureauOptions) -> Self {
		Self {
			bureaus: Vec::with_capacity(max),
			max,
			bureau_options,
		}
	}

	pub fn poll(&mut self) {
		self.bureaus.retain_mut(|bureau_ex| {
			bureau_ex.inner.poll();
			bureau_ex.start_time.elapsed().as_secs() < 10 || bureau_ex.inner.user_list.len() > 0
		})
	}

	pub fn available(&mut self) -> Option<u16> {
		if let Some(bureau_ex) = self.bureaus.iter().find(|bureau_ex| {
			bureau_ex.inner.user_list.len() < bureau_ex.inner.options.max_players as usize
		}) {
			return Some(bureau_ex.inner.port());
		}

		if self.bureaus.len() < self.max {
			let bureau = Bureau::new(Self::BIND_ADDR, self.bureau_options).ok()?;
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
