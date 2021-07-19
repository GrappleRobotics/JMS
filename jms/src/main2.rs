use schedule::{Annealer, ScheduleGenerator};

mod schedule;
mod logging;

fn main() {
  logging::configure(true);

  let gen = ScheduleGenerator::new(
    16, 10, 6
  );

  let anneal_team_balance = Annealer::new(1.0, 0.0, 100_000);
  let anneal_station_balance = Annealer::new(1.0, 0.0, 100_000);

  let (sched, tb, sb) = gen.generate(anneal_team_balance, anneal_station_balance);
  println!("TB: {}, SB: {}", tb, sb);
  println!("{}", sched.0.transpose());
}