#![allow(dead_code)]

mod bureau;
mod core;
mod wls;

use bureau::{Bureau, BureauOptions};
use wls::WLS;

fn main() {
	// WLS::start(5125).expect("Failed to start WLS server.");

	let bureau = match Bureau::new(
		"0.0.0.0:5126",
		BureauOptions {
			max_players: 256,
			aura_radius: 300.0,
		},
	) {
		Ok(v) => v,
		Err(_) => panic!("THE THING FAILED TO SPAWN!"),
	};

	bureau.join();
}
