use std::{num::ParseIntError, collections::HashMap};

use jms_base::kv;
use rand::Rng;

use crate::{db::Table, scoring::scores::{DerivedScore, WinStatus}};

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

impl Table for TeamRanking {
  const PREFIX: &'static str = "db:ranking";
  type Err = ParseIntError;
  type Id = usize;

  fn id(&self) -> Self::Id {
    self.team
  }
}

impl TeamRanking {
  pub fn update(kv: &kv::KVConnection) -> anyhow::Result<()> {
    let mut rankings_map = HashMap::new();

    let matches = super::Match::all_map(kv)?;
    let scores = super::CommittedMatchScores::all(kv)?;

    for score in scores {
      if let Some(m) = matches.get(&score.match_id) {
        if m.match_type == MatchType::Qualification {
          if let Some(score) = score.scores.last() {
            let score_red = score.red.derive(&score.blue);
            let score_blue = score.blue.derive(&score.red);

            for team in m.red_teams.iter().filter_map(|t| t.as_ref()) {
              Self::update_single(*team, &score_red, &mut rankings_map);
            }

            for team in m.blue_teams.iter().filter_map(|t| t.as_ref()) {
              Self::update_single(*team, &score_blue, &mut rankings_map);
            }
          }
        }
      }
    }

    Self::clear(kv)?;
    let r: anyhow::Result<Vec<()>> = rankings_map.into_values().map(|r| r.insert(kv)).collect();
    r?;
    Ok(())
  }

  fn update_single(team: usize, score: &DerivedScore, current_rankings: &mut HashMap<usize, TeamRanking>) {
    let mut rng = rand::thread_rng();
    let mut existing = current_rankings.get(&team).cloned().unwrap_or(TeamRanking {
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

    current_rankings.insert(team, existing);
  }

  pub fn sorted(db: &kv::KVConnection) -> anyhow::Result<Vec<Self>> {
    let mut v = Self::all(db)?;
    v.sort();
    Ok(v)
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
