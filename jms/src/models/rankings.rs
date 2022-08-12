use rand::Rng;

use crate::db::{self, TableType};
use crate::scoring::scores::{DerivedScore, WinStatus};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TeamRanking {
  pub team: usize,

  pub rp: usize,
  pub auto_points: isize,
  pub endgame_points: isize,
  pub teleop_points: isize,
  pub random_num: usize,

  pub win: usize,
  pub loss: usize,
  pub tie: usize,
  pub played: usize,
}

impl db::TableType for TeamRanking {
  const TABLE: &'static str = "rankings";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    Some(self.team.into())
  }
}

impl TeamRanking {
  pub fn get(team_num: usize, store: &db::Store) -> db::Result<TeamRanking> {
    let record = TeamRanking::table(store)?.get(team_num)?;
    match record {
      Some(rank) => Ok(rank),
      None => {
        let mut rng = rand::thread_rng();
        // Insert default
        let mut tr = TeamRanking {
          team: team_num,
          rp: 0, auto_points: 0,
          endgame_points: 0, teleop_points: 0,
          random_num: rng.gen(),
          win: 0, loss: 0, tie: 0, played: 0
        };

        tr.insert(store)?;
        Ok(tr)
      },
    }
  }

  pub fn update(
    &mut self,
    us_score: &DerivedScore,
    store: &db::Store,
  ) -> db::Result<()> {
    if us_score.win_status == WinStatus::WIN {
      self.rp += 2;
      self.win += 1;
    } else if us_score.win_status == WinStatus::LOSS {
      self.loss += 1;
    } else {
      self.rp += 1;
      self.tie += 1;
    }

    self.rp += us_score.total_bonus_rp;

    self.auto_points += us_score.mode_score.auto;
    self.teleop_points += us_score.mode_score.teleop;
    self.endgame_points += us_score.endgame_points;

    self.played += 1;

    self.insert(store)?;

    Ok(())
  }

  pub fn sorted(store: &db::Store) -> db::Result<Vec<TeamRanking>> {
    let all = Self::table(store)?.all();
    match all {
      Ok(mut als) => {
        als.sort();
        Ok(als)
      },
      Err(e) => Err(e),
    }
  }
}

fn cmp_f64(a: f64, b: f64) -> std::cmp::Ordering {
  if (a - b).abs() <= 1e-10 {
    std::cmp::Ordering::Equal
  } else {
    b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal)
  }
}

fn avg(x: isize, n: usize) -> f64 {
  (x as f64) / (n as f64)
}

impl PartialEq for TeamRanking {
  fn eq(&self, other: &Self) -> bool {
    self.team == other.team
  }
}

impl Eq for TeamRanking { }

impl PartialOrd for TeamRanking {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for TeamRanking {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    if self == other {
      return std::cmp::Ordering::Equal;
    }

    let n_self = self.played;
    let n_other = other.played;

    // Game Manual Table 11-2
    cmp_f64(avg(self.rp as isize, n_self), avg(other.rp as isize, n_other))
      .then(cmp_f64(avg(self.auto_points, n_self), avg(other.auto_points, n_other)))
      .then(cmp_f64(
        avg(self.endgame_points, n_self),
        avg(other.endgame_points, n_other),
      ))
      .then(cmp_f64(
        avg(self.teleop_points, n_self),
        avg(other.teleop_points, n_other),
      ))
      .then(cmp_f64(self.random_num as f64, other.random_num as f64))
  }
}
