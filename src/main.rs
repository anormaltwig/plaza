mod bureau;
mod lua_api;
mod math;
mod protocol;
mod user;
mod user_list;
mod wls;

use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bureau::{Bureau, BureauOptions};
use wls::{Wls, WlsOptions};

#[derive(Parser)]
struct Args {
	/// If set, branch will function in WLS mode.
	#[arg(short, long)]
	wls: bool,

	/// IP or Domain of server.
	#[arg(long, default_value_t = ("127.0.0.1").to_string())]
	host_name: String,

	/// Maximum number of bureaus to create in WLS mode.
	#[arg(long, default_value_t = 5)]
	max_bureaus: u32,

	/// Bureau/WLS port. (WLS will use this as a starting index for new bureaus)
	#[arg(short, long, default_value_t = 5126)]
	port: u16,

	/// Maximum number of users that each Bureau can have.
	#[arg(short, long, default_value_t = 256)]
	max_players: i32,

	/// Radius to add two users to each others aura.
	#[arg(short, long, default_value_t = 300.0)]
	aura_radius: f32,
}

fn main() {
	let args = Args::parse();

	let bureau_options = BureauOptions {
		max_players: args.max_players,
		aura_radius: args.aura_radius,
	};

	if args.wls {
		println!("Starting WLS on port {}", args.port);
		if let Err(io_err) = Wls::start(WlsOptions {
			max_bureaus: args.max_bureaus,
			host_name: args.host_name,
			port: args.port,
			bureau_options,
		}) {
			eprintln!("WLS failed to start. {}", io_err);
		}

		return;
	}

	println!("Starting Bureau on port {}", args.port);
	match Bureau::new(
		SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), args.port),
		bureau_options,
	) {
		Ok(handle) => match handle.join() {
			Ok(()) => (),
			Err(thread_err) => eprintln!("Bureau panicked! ({:?})", thread_err),
		},
		Err(io_err) => eprintln!("Bureau failed to start. {}", io_err),
	}
}
