mod arena;
mod db;
mod logging;
mod network;
mod utils;

mod models;
mod schema;

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use clap::{App, Arg};
use dotenv::dotenv;
use log::info;
use network::NetworkProvider;

struct FakeNetwork {}
impl NetworkProvider for FakeNetwork {
  fn configure_admin(&mut self) -> network::NetworkResult<()> {
    info!("Configuring Admin");
    Ok(())
  }

  fn configure_alliances(
    &mut self,
    stations: &mut dyn Iterator<Item = &arena::AllianceStation>,
    force_reload: bool,
  ) -> network::NetworkResult<()> {
    let alls: Vec<&arena::AllianceStation> = stations.collect();
    info!("Configuring Alliances (Force? {}): {:?}", force_reload, alls);
    Ok(())
  }
}

#[tokio::main]
async fn main() {
  dotenv().ok();

  let matches = App::new("JMS")
    .about("An Alternative Field-Management-System for FRC Offseason Events.")
    .arg(
      Arg::with_name("debug")
        .short("d")
        .help("Enable debug logging."),
    )
    .get_matches();

  logging::configure(matches.is_present("debug"));

  db::connection(); // Start connection

  let network = Box::new(FakeNetwork {});

  let mut arena = arena::Arena::new(3, network);
  log_expect!(arena.update());
}
