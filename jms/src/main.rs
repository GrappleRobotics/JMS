mod logging;
// mod network;
mod db;
mod utils;
mod arena;

mod models;
mod schema;

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use clap::{App, Arg};
use dotenv::dotenv;

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

  let mut arena = arena::Arena::new(3);
  log_expect!(arena.update());
}
