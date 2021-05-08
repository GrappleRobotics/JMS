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
extern crate strum;
#[macro_use]
extern crate strum_macros;

use clap::{App, Arg};
use dotenv::dotenv;
use log::info;
use network::NetworkProvider;

use crate::arena::matches::Match;

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
  arena.load_match(Match{state: arena::matches::MatchPlayState::Waiting});
  info!("{}", arena.current_state().variant());
  info!("{:?}", arena.queue_state_change(arena::ArenaStateVariant::MatchPlay));
  info!("{:?}", arena.perform_state_change());
  info!("{}", arena.current_state().variant());
  info!("{:?}", arena.queue_state_change(arena::ArenaStateVariant::PreMatch));
  info!("{:?}", arena.perform_state_change());
  info!("{}", arena.current_state().variant());

  let mut s = "".to_owned();

  while arena.current_state().variant() != arena::ArenaStateVariant::PreMatchComplete {
    match arena.maybe_queue_state_change(arena::ArenaStateVariant::PreMatchComplete) {
      Err(_) => { s = s + "."; Some(1) },
      Ok(()) => { arena.perform_state_change(); Some(1) }
    };
  }
  info!("{}", s);
  info!("{}", arena.current_state().variant());
}
