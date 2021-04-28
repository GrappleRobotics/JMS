mod logging;

use clap::{App, Arg};

fn main() {
  let matches = App::new("JMS")
                .about("An Alternative Field-Management-System for FRC Offseason Events.")
                .arg(Arg::with_name("debug")
                      .short("d")
                      .help("Enable debug logging."))
                .get_matches();

  logging::configure(matches.is_present("debug"));

  info!("In Root");
  context!("A", {
    debug!("A only... {}", 12);
    context!("Test1", "DHCP", {
      info!("Log 1");
    });
    debug!("A only... {}", 12);
    warn!("Oh No!");
    error!("AAAA");
  });
}