use clap::{Args, Parser, Subcommand};
use std::net::{IpAddr, SocketAddr};

use bureau::{Bureau, BureauConfig};
use wls::WlsOptions;

mod bureau;
mod wls;

#[derive(Parser)]
struct Cli {
	/// IP to bind to
	#[arg(long, default_value = "0.0.0.0")]
	ip: IpAddr,

	/// Port to bind to
	#[arg(short, long, default_value_t = 5126)]
	port: u16,

	#[command(flatten)]
	bureau: BureauArgs,

	#[command(subcommand)]
	command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
	/// Run a single bureau
	Bureau,
	/// Run in WLS mode (multiple bureaus for multiple wrls)
	Wls(WlsArgs),
}

#[derive(Args)]
struct BureauArgs {
	/// Amount of time to wait before disconnecting connecting users
	#[arg(short, long, default_value_t = 200.0)]
	aura_radius: f32,

	/// Amount of time to wait before disconnecting connecting users
	#[arg(long, default_value_t = 10)]
	connect_timeout: u64,

	/// Max players per bureau
	#[arg(short, long, default_value_t = 255)]
	max_users: i32,

	/// Max number of incoming connections to allow
	#[arg(long, default_value_t = 10)]
	max_queue: usize,
}

#[derive(Args)]
struct WlsArgs {
	/// Host name or IP address to use for WLS responses
	#[arg(short = 'n', long, default_value_t = ("127.0.0.1").to_string())]
	host_name: String,

	/// Max bureaus per wrl
	#[arg(short, long, default_value_t = 2)]
	max_bureaus: usize,

	/// Max players per bureau
	#[arg(long)]
	wrl_list: Option<String>,
}

fn main() {
	let cli = Cli::parse();

	let addr = SocketAddr::new(cli.ip, cli.port);

	let bureau_config = BureauConfig {
		connect_timeout: cli.bureau.connect_timeout,
		max_users: cli.bureau.max_users,
		max_queue: cli.bureau.max_queue,
		aura_radius: cli.bureau.aura_radius,
		wrl: None,
	};

	match cli.command {
		CliCommand::Bureau => {
			println!("Running Bureau on port '{}.'", cli.port);

			Bureau::new(addr, bureau_config)
				.expect("bureau creation")
				.run();
		}
		CliCommand::Wls(args) => {
			wls::run(
				addr,
				WlsOptions {
					host_name: args.host_name,
					max_bureaus: args.max_bureaus,
					wrl_list: args.wrl_list,
					bureau_config,
				},
			)
			.expect("running wls");
		}
	}
}
