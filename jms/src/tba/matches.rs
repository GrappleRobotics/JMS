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
  pub score_breakdown: Option<TBA2021ScoreBreakdownFull>,
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

    let score_breakdown = score.map(|score| TBA2021ScoreBreakdownFull::from(score));

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
pub struct TBA2021ScoreBreakdownFull {
  blue: TBA2021ScoreBreakdown,
  red: TBA2021ScoreBreakdown,
}

impl From<MatchScoreSnapshot> for TBA2021ScoreBreakdownFull {
  fn from(score: MatchScoreSnapshot) -> Self {
    Self {
      red: score.red.into(),
      blue: score.blue.into()
    }
  }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct TBA2021ScoreBreakdown {
  // Thanks Cheesy-Arena :)
  initLineRobot1: &'static str,
  initLineRobot2: &'static str,
  initLineRobot3: &'static str,
  autoCellsBottom: usize,
  autoCellsOuter: usize,
  autoCellsInner: usize,
  teleopCellsBottom: usize,
  teleopCellsOuter: usize,
  teleopCellsInner: usize,
  stage1Activated: bool,
  stage2Activated: bool,
  stage3Activated: bool,
  stage3TargetColor: &'static str,
  endgameRobot1: &'static str,
  endgameRobot2: &'static str,
  endgameRobot3: &'static str,
  endgameRungIsLevel: &'static str,
  foulCount: usize,
  techFoulCount: usize,
  autoInitLinePoints: isize,
  autoCellPoints: isize,
  autoPoints: isize,
  teleopCellPoints: isize,
  controlPanelPoints: isize,
  endgamePoints: isize,
  teleopPoints: isize,
  foulPoints: usize,
  totalPoints: usize,
  shieldEnergizedRankingPoint: bool,
  shieldOperationalRankingPoint: bool,
  rp: usize
}

impl TBA2021ScoreBreakdown {
  pub fn init_crossed_str(crossed: Option<&bool>) -> &'static str {
    crossed.and_then(|c| c.then(|| "Exited")).unwrap_or("None")
  }

  pub fn endgame_str(endgame: Option<&EndgamePointType>) -> &'static str {
    match endgame {
      Some(EndgamePointType::None) => "None",
      Some(EndgamePointType::Park) => "Park",
      Some(EndgamePointType::Hang) => "Hang",
      None => "None",
    }
  }
}

impl From<SnapshotScore> for TBA2021ScoreBreakdown {
  fn from(score: SnapshotScore) -> Self {
    let live = score.live;
    let derived = score.derived;

    TBA2021ScoreBreakdown {
      initLineRobot1: TBA2021ScoreBreakdown::init_crossed_str(live.initiation_line_crossed.get(0)),
      initLineRobot2: TBA2021ScoreBreakdown::init_crossed_str(live.initiation_line_crossed.get(1)),
      initLineRobot3: TBA2021ScoreBreakdown::init_crossed_str(live.initiation_line_crossed.get(2)),
      autoCellsBottom: live.power_cells.auto.bottom,
      autoCellsOuter: live.power_cells.auto.outer,
      autoCellsInner: live.power_cells.auto.inner,
      teleopCellsBottom: live.power_cells.teleop.bottom,
      teleopCellsOuter: live.power_cells.teleop.outer,
      teleopCellsInner: live.power_cells.teleop.inner,
      stage1Activated: derived.stage >= 1,
      stage2Activated: derived.stage >= 2,
      stage3Activated: derived.stage >= 3,
      stage3TargetColor: "Unknown",
      endgameRobot1: TBA2021ScoreBreakdown::endgame_str(live.endgame.get(0)),
      endgameRobot2: TBA2021ScoreBreakdown::endgame_str(live.endgame.get(1)),
      endgameRobot3: TBA2021ScoreBreakdown::endgame_str(live.endgame.get(2)),
      endgameRungIsLevel: live.rung_level.then(|| "IsLevel").unwrap_or("NotLevel"),
      foulCount: live.penalties.fouls,
      techFoulCount: live.penalties.tech_fouls,
      autoInitLinePoints: derived.initiation_points,
      autoCellPoints: derived.cell_points.auto,
      autoPoints: derived.mode_score.auto,
      teleopCellPoints: derived.cell_points.teleop,
      controlPanelPoints: 0,
      endgamePoints: derived.endgame_points,
      teleopPoints: derived.mode_score.teleop,
      foulPoints: derived.penalty_score,
      totalPoints: derived.total_score,
      shieldEnergizedRankingPoint: derived.stage3_rp,
      shieldOperationalRankingPoint: derived.shield_gen_rp,
      rp: derived.total_rp,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::convert::TryInto;
  use crate::{db::DBDateTime, scoring::scores::{LiveScore, MatchScore, ModeScore, Penalties, PowerCellCounts}};


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