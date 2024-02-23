mod command_interface;
mod service;

pub fn setup_logging() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    nym_vpn_lib::nym_config::defaults::setup_env(Some("/home/jon/src/nym/nym/envs/sandbox.env"));

    // The idea here for explicly starting two separate runtimes is to make sure they are properly
    // separated. Looking ahead a little ideally it would be nice to be able for the command
    // interface to be able to forcefully terminate the vpn if needed.

    println!("main: starting command handler");
    let (command_handle, vpn_command_rx) = command_interface::start_command_interface();

    println!("main: starting VPN handler");
    let vpn_handle = service::start_vpn_service(vpn_command_rx);

    command_handle.join().unwrap();
    vpn_handle.join().unwrap();

    Ok(())
}
