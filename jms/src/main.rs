mod logging;
// mod network;
mod utils;
mod db;

mod models;
mod schema;

#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate diesel;

use std::{thread, time::Duration};

use clap::{App, Arg};
use diesel::prelude::*;
use dotenv::dotenv;
use log::info;
use rand::Rng;

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

  let conn = db::connection();

  // let team = models::Team { id: 5333, name: String::from("Can't C#"), affiliation: Some(String::from("Curtin University")), location: None, notes: None };
  // info!("{:?}", log_expect!(diesel::insert_into(schema::teams::table).values(&team).execute(&conn)));
  
  diesel::delete(schema::teams::table).execute(&conn);
  for _ in 1..20 {
    thread::spawn(|| {
      let team = models::Team { id: rand::thread_rng().gen(), name: String::from("Some Team"), affiliation: None, location: None, notes: None };
      log_expect!(diesel::insert_into(schema::teams::table).values(&team).execute(&db::connection()));
    });
  }

  thread::sleep(Duration::from_secs(5));
  info!("{:?}", schema::teams::table.load::<models::Team>(&conn));
  // log_expect!(danger_or_err());

  // let handle = log_expect!(network::onboard::netlink::handle());
  // let addrs = vec!["10.0.100.1", "10.0.100.5"]
  //   .into_iter()
  //   .map(|x| x.parse().unwrap());
  // log_expect!(configure_addresses(handle, "ens19.100", addrs).await);
}
