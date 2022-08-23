use std::collections::HashMap;

use rand::Rng;

use crate::db::{self, TableType};
use crate::scoring::scores::{DerivedScore, WinStatus};

use super::MatchType;

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
  // Update the rankings cache whenever a new match is added / removed
  pub async fn run() -> anyhow::Result<()> {
    let mut watch = super::Match::table(&db::database())?.watch_all();
    loop {
      let _event = watch.get().await?;
      Self::update()?;
    }
  }

  pub fn update() -> db::Result<()> {
    let mut rmap = HashMap::new();
    for m in super::Match::all(&db::database())? {
      if m.played && m.match_type == MatchType::Qualification {
        if let Some(score) = m.score {
          let score_red = score.red.derive(&score.blue);
          let score_blue = score.blue.derive(&score.red);

          for team in m.red_teams.into_iter().filter_map(|t| t) {
            Self::update_single(team, &score_red, &mut rmap);
          }

          for team in m.blue_teams.into_iter().filter_map(|t| t) {
            Self::update_single(team, &score_blue, &mut rmap);
          }
        }
      }
    }

    Self::clear(&db::database())?;
    let r: db::Result<Vec<()>> = rmap.into_values().map(|mut r| r.insert(&db::database()).map(|_| ())).collect();
    r?;
    Ok(())
  }

  fn update_single(team: usize, score: &DerivedScore, current: &mut HashMap<usize, TeamRanking>) {
    let mut rng = rand::thread_rng();
    let mut existing = current.get(&team).cloned().unwrap_or(TeamRanking {
      team, rp: 0, auto_points: 0,
      endgame_points: 0, teleop_points: 0,
      random_num: rng.gen(),
      win: 0, loss: 0, tie: 0, played: 0
    });

    existing.rp += score.total_bonus_rp;
    existing.auto_points += score.mode_score.auto;
    existing.teleop_points += score.mode_score.teleop;
    existing.endgame_points += score.endgame_points;
    existing.played += 1;

    match score.win_status {
      WinStatus::WIN => {
        existing.rp += 2;
        existing.win += 1;
      },
      WinStatus::LOSS => {
        existing.loss += 1;
      },
      WinStatus::TIE => {
        existing.rp += 1;
        existing.tie += 1;
      },
    }

    current.insert(team, existing);
  }

  pub fn sorted(store: &db::Store) -> db::Result<Vec<TeamRanking>> {
    match Self::all(store) {
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
