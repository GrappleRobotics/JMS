use std::convert::TryFrom;

use chrono::{Local, TimeZone, Utc};

use crate::{models, scoring::scores::{EndgamePointType, MatchScoreSnapshot, SnapshotScore}};

use super::{TBAClient, teams::TBATeam};

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TBAMatchAlliance {
  teams: Vec<TBATeam>,
  score: Option<isize>,
  // We don't do surrogates and DQs
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TBAMatchAlliances {
  red: TBAMatchAlliance,
  blue: TBAMatchAlliance,
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct TBAMatchLevel(&'static str);

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TBAMatch {
  pub comp_level: TBAMatchLevel,
  pub set_number: usize,
  pub match_number: usize,
  pub alliances: TBAMatchAlliances,
  pub score_breakdown: Option<TBA2022ScoreBreakdownFull>,
  pub time_str: Option<String>,
  pub time_utc: Option<String>
}

impl TryFrom<(models::MatchType, Option<models::MatchSubtype>)> for TBAMatchLevel {
  type Error = anyhow::Error;

  fn try_from((mt, mst): (models::MatchType, Option<models::MatchSubtype>)) -> anyhow::Result<TBAMatchLevel> {
    Ok(match mt {
      models::MatchType::Qualification => TBAMatchLevel("qm"),
      models::MatchType::Playoff => match mst {
        Some(models::MatchSubtype::Quarterfinal) => TBAMatchLevel("qf"),
        Some(models::MatchSubtype::Semifinal) => TBAMatchLevel("sf"),
        Some(models::MatchSubtype::Final) => TBAMatchLevel("f"),
        None => anyhow::bail!("Playoff matches must have a subtype!"),
      },
      _ => anyhow::bail!("{:?} is an invalid match type", mt)
    })
  }
}

impl TryFrom<models::Match> for TBAMatch {
  type Error = anyhow::Error;

  fn try_from(m: models::Match) -> anyhow::Result<TBAMatch> {
    let time = m.score_time.or(m.start_time).map(|t| t.0);
    let score: Option<MatchScoreSnapshot> = m.score.map(|ms| ms.into());

    let alliances = TBAMatchAlliances {
      red: TBAMatchAlliance {
        score: score.as_ref().map(|s| s.red.derived.total_score as isize),
        teams: m.red_teams.iter().filter_map(|&t| t.map(|tn| TBATeam::from(tn as usize) )).collect()
      },
      blue: TBAMatchAlliance {
        score: score.as_ref().map(|s| s.blue.derived.total_score as isize),
        teams: m.blue_teams.iter().filter_map(|&t| t.map(|tn| TBATeam::from(tn as usize) )).collect()
      }
    };

    let score_breakdown = score.map(|score| TBA2022ScoreBreakdownFull::from(score));

    Ok(TBAMatch {
      comp_level: TBAMatchLevel::try_from(( m.match_type, m.match_subtype ))?,
      set_number: m.set_number as usize,
      match_number: m.match_number as usize,
      alliances,
      score_breakdown,
      time_str: time.map(|t| Local.from_utc_datetime(&t).format("%l:%M %p").to_string()),
      time_utc: time.map(|t| Utc.from_utc_datetime(&t).format("%+").to_string())
    })
  }
}

impl TBAMatch {
  pub async fn delete(code: String, client: &TBAClient) -> anyhow::Result<()> {
    client.post("matches", "delete", &vec![code]).await
  }

  pub async fn issue(&self, client: &TBAClient) -> anyhow::Result<()> {
    client.post("matches", "update", &vec![&self]).await
  }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TBA2022ScoreBreakdownFull {
  blue: TBA2022ScoreBreakdown,
  red: TBA2022ScoreBreakdown,
}

impl From<MatchScoreSnapshot> for TBA2022ScoreBreakdownFull {
  fn from(score: MatchScoreSnapshot) -> Self {
    Self {
      red: score.red.into(),
      blue: score.blue.into()
    }
  }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct TBA2022ScoreBreakdown {
  // Thanks Cheesy-Arena :)
  taxiRobot1: &'static str,
  taxiRobot2: &'static str,
  taxiRobot3: &'static str,
  autoCargoLowerBlue: usize,
  autoCargoLowerRed: usize,
  autoCargoLowerFar: usize,
  autoCargoLowerNear: usize,
  autoCargoUpperBlue: usize,
  autoCargoUpperRed: usize,
  autoCargoUpperFar: usize,
  autoCargoUpperNear: usize,
  autoCargoTotal: usize,
  teleopCargoLowerBlue: usize,
  teleopCargoLowerRed: usize,
  teleopCargoLowerFar: usize,
  teleopCargoLowerNear: usize,
  teleopCargoUpperBlue: usize,
  teleopCargoUpperRed: usize,
  teleopCargoUpperFar: usize,
  teleopCargoUpperNear: usize,
  teleopCargoTotal: usize,
  matchCargoTotal: usize,
  endgameRobot1: &'static str,
  endgameRobot2: &'static str,
  endgameRobot3: &'static str,
  autoTaxiPoints: usize,
  autoCargoPoints: usize,
  teleopCargoPoints: usize,
  autoPoints: usize,
  endgamePoints: usize,
  teleopPoints: usize,
  quintetAchieved: bool,
  cargoBonusRankingPoint: bool,
  hangarBonusRankingPoint: bool,
  foulCount: usize,
  techFoulCount: usize,
  foulPoints: usize,
  totalPoints: usize,
  rp: usize
}

impl TBA2022ScoreBreakdown {
  pub fn taxi_str(taxi: Option<&bool>) -> &'static str {
    taxi.and_then(|c| c.then(|| "Yes")).unwrap_or("No")
  }

  pub fn endgame_str(endgame: Option<&EndgamePointType>) -> &'static str {
    match endgame {
      Some(EndgamePointType::None) => "None",
      Some(EndgamePointType::Low) => "Low",
      Some(EndgamePointType::Mid) => "Mid",
      Some(EndgamePointType::High) => "High",
      Some(EndgamePointType::Traversal) => "Traversal",
      None => "None",
    }
  }
}

impl From<SnapshotScore> for TBA2022ScoreBreakdown {
  fn from(score: SnapshotScore) -> Self {
    let live = score.live;
    let derived = score.derived;

    TBA2022ScoreBreakdown {
      taxiRobot1: Self::taxi_str(live.taxi.get(0)),
      taxiRobot2: Self::taxi_str(live.taxi.get(1)),
      taxiRobot3: Self::taxi_str(live.taxi.get(2)),
      autoCargoLowerBlue: live.cargo.auto.lower[0],
      autoCargoLowerRed: live.cargo.auto.lower[2],
      autoCargoLowerFar: live.cargo.auto.lower[3],
      autoCargoLowerNear: live.cargo.auto.lower[1],
      autoCargoUpperBlue: live.cargo.auto.upper[0],
      autoCargoUpperRed: live.cargo.auto.upper[2],
      autoCargoUpperFar: live.cargo.auto.upper[3],
      autoCargoUpperNear: live.cargo.auto.upper[1],
      autoCargoTotal: live.cargo.auto.total_cargo(),
      teleopCargoLowerBlue: live.cargo.teleop.lower[0],
      teleopCargoLowerRed: live.cargo.teleop.lower[2],
      teleopCargoLowerFar: live.cargo.teleop.lower[3],
      teleopCargoLowerNear: live.cargo.teleop.lower[1],
      teleopCargoUpperBlue: live.cargo.teleop.upper[0],
      teleopCargoUpperRed: live.cargo.teleop.upper[2],
      teleopCargoUpperFar: live.cargo.teleop.upper[3],
      teleopCargoUpperNear: live.cargo.teleop.upper[1],
      teleopCargoTotal: live.cargo.teleop.total_cargo(),
      matchCargoTotal: live.cargo.total().total_cargo(),
      endgameRobot1: Self::endgame_str(live.endgame.get(0)),
      endgameRobot2: Self::endgame_str(live.endgame.get(1)),
      endgameRobot3: Self::endgame_str(live.endgame.get(2)),
      autoTaxiPoints: derived.taxi_points as usize,
      autoCargoPoints: derived.cargo_points.auto as usize,
      teleopCargoPoints: derived.cargo_points.teleop as usize,
      autoPoints: derived.mode_score.auto as usize,
      endgamePoints: derived.endgame_points as usize,
      teleopPoints: derived.mode_score.teleop as usize,
      quintetAchieved: derived.quintet,
      cargoBonusRankingPoint: derived.cargo_rp,
      hangarBonusRankingPoint: derived.hangar_rp,
      foulCount: live.penalties.fouls,
      techFoulCount: live.penalties.tech_fouls,
      foulPoints: derived.penalty_score,
      totalPoints: derived.total_score,
      rp: derived.total_rp,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::convert::TryInto;
  use crate::db::DBDateTime;


  #[test]
  pub fn test_matches_without_score() -> anyhow::Result<()> {
    let dt = Utc.ymd(2021, 08, 17).and_hms(9, 10, 11);
    let test_cases = vec![
      (models::MatchType::Qualification, None, "qm"),
      (models::MatchType::Playoff, Some(models::MatchSubtype::Quarterfinal), "qf"),
      (models::MatchType::Playoff, Some(models::MatchSubtype::Semifinal), "sf"),
      (models::MatchType::Playoff, Some(models::MatchSubtype::Final), "f"),
    ];

    for (match_type, match_subtype, comp_level) in test_cases {
      let m = models::Match {
        start_time: Some(DBDateTime(dt.naive_utc())),
        match_type,
        set_number: 1,
        match_number: 1,
        blue_teams: vec![ Some(113), None, Some(112) ],
        red_teams: vec![None, Some(4788), None],
        played: true,
        score: None,
        winner: None,
        match_subtype,
        red_alliance: None,
        blue_alliance: None,
        score_time: None,
      };
  
      let tba_match: TBAMatch = m.try_into()?;

      assert_eq!(tba_match.comp_level, TBAMatchLevel(comp_level));
      assert_eq!(tba_match.set_number, 1);
      assert_eq!(tba_match.match_number, 1);
      assert_eq!(tba_match.alliances.red.teams, vec![ TBATeam::from(4788) ]);
      assert_eq!(tba_match.alliances.blue.teams, vec![ TBATeam::from(113), TBATeam::from(112) ]);
      assert_eq!(tba_match.score_breakdown, None);
      assert_eq!(tba_match.time_utc, Some("2021-08-17T09:10:11+00:00".to_owned()));
    }

    Ok(())
  }
}