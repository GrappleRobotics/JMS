mod logging;
mod network;
mod utils;

use clap::{App, Arg};

use crate::network::onboard::netlink::configure_addresses;

#[tokio::main]
async fn main() {
  let matches = App::new("JMS")
    .about("An Alternative Field-Management-System for FRC Offseason Events.")
    .arg(
      Arg::with_name("debug")
        .short("d")
        .help("Enable debug logging."),
    )
    .get_matches();

  logging::configure(matches.is_present("debug"));

  // log_expect!(danger_or_err());

  let handle = log_expect!(network::onboard::netlink::handle());
  let addrs = vec!["10.0.100.1", "10.0.100.5"]
    .into_iter()
    .map(|x| x.parse().unwrap());
  log_expect!(configure_addresses(handle, "ens19.100", addrs).await);
}
