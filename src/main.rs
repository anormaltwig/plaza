mod bureau;
mod lua_api;
mod math;
mod protocol;
mod user;
mod user_list;
mod wls;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bureau::{Bureau, BureauOptions};
use clap::Parser;

#[derive(Parser)]
struct Args {
	#[arg(short, long)]
	wls: bool,

	#[arg(short, long, default_value_t = 5126)]
	port: u16,

	#[arg(short, long, default_value_t = 256)]
	maxplayers: i32,

	#[arg(short, long, default_value_t = 300.0)]
	auraradius: f32,
}

fn main() {
	let args = Args::parse();

	if args.wls {
		unimplemented!("oops no wls.");
	} else {
		println!("Starting Bureau on port {}", args.port);

		match Bureau::new(
			SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), args.port),
			BureauOptions {
				max_players: args.maxplayers,
				aura_radius: args.auraradius,
			},
		) {
			Ok(v) => v,
			Err(_) => panic!("Failed to start Bureau."),
		}
		.join()
		.expect("Error while joining Bureau thread");
	}
}
