use std::{cmp, time};

use log::{debug, info};
use nalgebra as na;
use rand::{Rng, prelude::SliceRandom};

pub mod quals;

// Cols = Rounds
#[derive(Debug)]
pub struct ScheduleRounds(pub na::DMatrix<usize>);

// Cols = Matches
#[derive(Debug)]
pub struct Schedule(pub na::DMatrix<usize>);

impl Schedule {
  pub fn contextualise(&self, teams: &[u16]) -> TeamSchedule {
    TeamSchedule(self.0.map(|x| teams[x]))
  }
}

#[derive(Debug)]
pub struct TeamSchedule(pub na::DMatrix<u16>);

pub struct ScheduleGenerator {
  num_teams: usize,
  num_rounds: usize,
  num_stations: usize,

  num_matches: usize,
  num_overflow: usize,

  teams: na::DVector<usize>,

  placeholder_team: usize
}

impl ScheduleGenerator {
  pub fn new(
    num_teams: usize,
    // num_matches_per_team: usize,
    num_matches: usize,
    num_stations: usize
  ) -> Self {
    // let num_matches = ((num_teams * num_matches_per_team) as f64 / (num_stations as f64)).ceil() as usize;
    let num_matches_per_team = (num_matches * num_stations) / (num_teams);
    let teams = na::DVector::from_iterator(num_teams, (0..num_teams).into_iter());

    Self {
      num_teams,
      num_rounds: num_matches_per_team,
      num_stations,
      num_matches,
      num_overflow: (num_matches * num_stations) - (num_matches_per_team * num_teams),
      teams,
      placeholder_team: num_teams
    }
  }

  fn generate_simple_schedule(&self) -> Schedule {
    let mut schedule = na::DMatrix::zeros(self.num_stations, self.num_matches);

    let mut teams: Vec<usize> = shuffle(&self.teams).iter().map(|&x| x).collect();

    for m in 0..self.num_matches {
      let mut match_picked = vec![];
      for s in 0..self.num_stations {
        if teams.len() == 0 {
          teams = shuffle(&self.teams).iter().map(|&x| x).collect();
        }

        // Get the first team in the shuffled teams that isn't already in this match 
        // (for when rounds split in the middle of a match)
        let pos = teams.iter().position(|&x| !match_picked.iter().any(|&y| x == y)).unwrap();
        let picked = teams.remove(pos);
        match_picked.push(picked);

        schedule[(s, m)] = picked;
      }
    }

    Schedule(schedule)
  }

  // fn matches_into_rounds(&self, matches: &Schedule) -> ScheduleRounds {
  //   ScheduleRounds(na::DMatrix::from_iterator(self.num_teams, self.num_rounds, matches.0.iter().cloned()))
  // }

  pub fn schedule_team_balance_score(&self, schedule: &Schedule) -> Option<f64> {
    // Generate cooccurence of each team with each other
    let cooccurrence = self.cooccurrence_matrix(&schedule);

    // Calculate stddev of upper triangle (first part of the cooccurrence)
    cooccurrence.map(|c| stddev(&upper_triangle(&c, 1)))
  }

  pub fn cooccurrence_matrix(&self, schedule: &Schedule) -> Option<na::DMatrix<usize>> {
    let mut cooccurrence: na::DMatrix<usize> = na::DMatrix::zeros(self.num_teams, self.num_teams);

    for m in schedule.0.column_iter() {
      for (i, &t1) in m.iter().enumerate() {
        for (j, &t2) in m.iter().enumerate() {
          if t1 != self.placeholder_team && t2 != self.placeholder_team {
            if i != j && t1 == t2 {
              // Team appears in the same match multiple times, therefore this schedule is not valid
              return None
            } else {
              cooccurrence[(t1, t2)] += 1;
            }
          }
        }
      }
    }
    
    Some(cooccurrence)
  }

  // TODO: Make this shuffle properly
  // TODO: Need to select a round and make sure matchups are still valid
  // pub fn generate_incremental_team_balance_schedule(&self, schedule: &Schedule) -> Schedule {
  //   let mut rng = rand::thread_rng();

  //   let mut sched = schedule.0.clone();
  //   let col = rng.gen_range(0..sched.ncols());
  //   sched.set_column(col, &shuffle(&sched.column(col)));
  //   ScheduleRounds(sched)
  // }

  pub fn schedule_station_balance_scores(&self, schedule: &Schedule) -> Option<f64> {
    // Each row is a station, each col is a team
    let stations = self.station_matrix(schedule);
    
    // Get the stddevs of each team, and then average together
    let stddevs = stations.column_iter().map(|ref x| stddev(x));
    Some(stddevs.clone().sum::<f64>() / (stddevs.len() as f64))
  }

  pub fn station_matrix(&self, schedule: &Schedule) -> na::DMatrix<usize> {
    let mut stations: na::DMatrix<usize> = na::DMatrix::zeros(self.num_stations, self.num_teams);
    for c in schedule.0.column_iter() {
      for (stn_i, &team) in c.iter().enumerate() {
        stations[(stn_i, team)] += 1;
      }
    }
    stations
  }

  pub fn generate_incremental_station_balance_schedule(&self, schedule: &Schedule) -> Schedule {
    let mut rng = rand::thread_rng();

    let mut sched = schedule.0.clone();
    let col = rng.gen_range(0..sched.ncols());
    sched.set_column(col, &shuffle(&sched.column(col)));
    Schedule(sched)
  }

  pub fn generate(&self, anneal_team_balance: Annealer, anneal_station_balance: Annealer) -> (Schedule, f64, f64) {
    let t0 = time::Instant::now();

    let seed = self.generate_simple_schedule();
    let t1 = time::Instant::now();

    info!("Seed schedule generated (in {:.2}s)", (t1 - t0).as_secs_f32());
    debug!("{}", seed.0);

    let t2 = time::Instant::now();
    let (annealed_1, tb_initial_score, tb_score) = anneal_team_balance.anneal(
      seed,
      // |s| self.generate_incremental_team_balance_schedule(s),
      |_| self.generate_simple_schedule(),
      |s| self.schedule_team_balance_score(s)
    );
    let t3 = time::Instant::now();

    info!("Team balance annealing complete, score={:.4}->{:.4} (in {:.2}s)", tb_initial_score, tb_score, (t3 - t2).as_secs_f32());
    debug!("{}", annealed_1.0);

    // let matches = self.rounds_into_matches(&annealed_1, false);

    let t4 = time::Instant::now();
    let (annealed_2, sb_initial_score, sb_score) = anneal_station_balance.anneal(
      annealed_1,
      |s| self.generate_incremental_station_balance_schedule(s),
      |s| self.schedule_station_balance_scores(s)
    );
    let t5 = time::Instant::now();

    info!("Station balance annealing complete, score={:.4}->{:.4} (in {:.2}s)", sb_initial_score, sb_score, (t5 - t4).as_secs_f32());
    debug!("{}", annealed_2.0);

    info!("Schedule generated in {:.2}s", (t5 - t0).as_secs_f32());
    debug!("{}", annealed_2.0);

    debug!("Cooccurrence matrix:");
    debug!("{}", self.cooccurrence_matrix(&annealed_2).unwrap());

    debug!("Station matrix:");
    debug!("{}", self.station_matrix(&annealed_2));

    (annealed_2, tb_score, sb_score)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Annealer {
  temp_start: f64,
  temp_end: f64,
  dt: f64
}

impl Annealer {
  pub fn new(temp_start: f64, temp_end: f64, steps: usize) -> Self {
    Annealer {
      temp_start,
      temp_end,
      dt: (temp_start - temp_end) / (steps as f64)
    }
  }

  pub fn anneal<T, G, E>(&self, initial: T, generator: G, evaluator: E) -> (T, f64, f64)
  where
    G: Fn(&T) -> T,
    E: Fn(&T) -> Option<f64>
  {
    let mut rng = rand::thread_rng();

    let mut temperature = self.temp_start;
    let mut current = initial;
    let mut current_score = evaluator(&current).unwrap();
    let initial_score = current_score;

    while temperature > self.temp_end {
      let next = generator(&current);

      if let Some(score) = evaluator(&next) {
        let prob = f64::exp(-(score - current_score) / temperature);
        let roll: f64 = rng.gen();
        if score < current_score || roll <= prob {
          current = next;
          current_score = score;
        }
      }

      temperature -= self.dt;
    }

    (current, initial_score, current_score)
  }
}

// Allow any storage - slice or vec
fn shuffle<S>(mat: &na::Matrix<usize, na::Dynamic, na::U1, S>) -> na::DVector<usize>
where
  S: na::storage::Storage<usize, na::Dynamic, na::U1>
{
  let mut rng = rand::thread_rng();
  let mut x: Vec<usize> = mat.into_iter().map(|x| *x).collect();
  x.shuffle(&mut rng);
  na::DVector::from_vec(x)
}

fn stddev<S>(mat: &na::Matrix<usize, na::Dynamic, na::U1, S>) -> f64
where
  S: na::storage::Storage<usize, na::Dynamic, na::U1>
{
  let floating = mat.map(|x| x as f64);
  let mean = floating.mean();
  let numer = floating.map(|x| (x - mean).powi(2)).sum();
  (numer / (mat.len() as f64)).sqrt()
}

fn upper_triangle(mat: &na::DMatrix<usize>, shift: usize) -> na::DVector<usize> {
  let mut upper = na::DVector::zeros(0);
  for j in shift..mat.ncols() {
    for i in 0..cmp::min(j + 1 - shift, mat.nrows()) {
      let n = upper.len();
      upper = upper.insert_row(n, mat[(i, j)]);
    }
  }
  upper
}