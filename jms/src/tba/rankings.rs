use crate::models;

use super::teams::TBATeam;

#[derive(serde::Serialize, Debug, Clone)]
pub struct TBATeamRank {
  team_key: TBATeam,
  rank: usize,
  wins: usize,
  losses: usize,
  ties: usize,
  played: usize,
  dqs: usize,

  rp: f64,
  auto: isize,
  teleop: isize,
  endgame: isize,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct TBARankings {
  breakdowns: Vec<&'static str>,
  rankings: Vec<TBATeamRank>
}

impl From<Vec<models::TeamRanking>> for TBARankings {
  fn from(mut ranks: Vec<models::TeamRanking>) -> Self {
    ranks.sort();
    let breakdowns = vec!["wins", "losses", "ties", "rp", "auto", "endgame", "teleop"];
    let rankings = ranks.iter().enumerate().map(|(i, r)| TBATeamRank {
      team_key: TBATeam::from(r.team as usize),
      rank: i,
      wins: r.win as usize,
      losses: r.loss as usize,
      ties: r.tie as usize,
      played: r.played as usize,
      dqs: 0,
      rp: (r.rp as f64) / (r.played as f64),
      auto: r.auto_points as isize,
      teleop: r.teleop_points as isize,
      endgame: r.endgame_points as isize
    }).collect();

    Self {
      breakdowns, rankings
    }
  }
}
