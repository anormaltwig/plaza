mod bureau;
mod wls;

use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::{
	bureau::{Bureau, BureauOptions},
	wls::WlsOptions,
};

#[derive(Parser)]
struct Args {
	/// If set, program will function in WLS mode.
	#[arg(short, long)]
	wls: bool,

	/// IP or Domain of the server.
	#[arg(long, default_value_t = ("127.0.0.1").into())]
	host_name: String,

	/// Maximum number of bureaus per wrl to create in WLS mode.
	#[arg(long, default_value_t = 3)]
	max_bureaus: usize,

	/// File path to a newline seperated list of wrls to allow in WLS mode.
	#[arg(long)]
	wrl_list: Option<String>,

	/// Bureau/WLS port.
	#[arg(short, long, default_value_t = 5126, value_parser = clap::value_parser!(u16).range(1..))]
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

	let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), args.port);

	if args.wls {
		// Never returns unless it errors.
		let err = wls::run(
			bind_addr,
			WlsOptions {
				host_name: args.host_name,
				max_bureaus: args.max_bureaus,
				wrl_list: args.wrl_list,
				bureau_options,
			},
		)
		.unwrap_err();

		eprintln!("Failed to run WLS: {}", err);

		return;
	}

	let mut bureau = match Bureau::new(bind_addr, bureau_options) {
		Ok(bureau) => bureau,
		Err(err) => {
			eprintln!("Failed to run Bureau: '{}'.", err);

			return;
		}
	};

	println!("Bureau running on port: {}.", bureau.port());
	bureau.run();
}
