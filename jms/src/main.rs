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

use std::{thread, time::Duration};

use clap::{App, Arg};
use dotenv::dotenv;
use log::info;
use network::NetworkProvider;

use crate::arena::{ArenaSignal, ArenaState, matches::Match};

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
    info!(
      "Configuring Alliances (Force? {}): {:?}",
      force_reload, alls
    );
    thread::sleep(Duration::from_millis(1000));
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

  let mut arena = arena::Arena::new(3, Some(network));
  arena.load_match(Match { state: arena::matches::MatchPlayState::Waiting });
  arena.update();
  assert_eq!(arena.current_state(), ArenaState::Idle);
  arena.signal(ArenaSignal::Prestart(false));
  arena.update();
  assert_eq!(arena.current_state(), ArenaState::Prestart(false, false));
  let mut s= "".to_owned();
  while let ArenaState::Prestart(false, _) = arena.current_state() {
    arena.update();
    s = s + ".";
    thread::sleep(Duration::from_millis(10));
  }
  assert_eq!(arena.current_state(), ArenaState::Prestart(true, false));
}
