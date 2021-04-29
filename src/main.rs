mod logging;
mod utils;

use clap::{App, Arg};

fn main() {
  let matches = App::new("JMS")
                .about("An Alternative Field-Management-System for FRC Offseason Events.")
                .arg(Arg::with_name("debug")
                      .short("d")
                      .help("Enable debug logging."))
                .get_matches();

  logging::configure(matches.is_present("debug"));

  info!("In danger? {}", *utils::danger::IS_DANGER_ZONE);
}