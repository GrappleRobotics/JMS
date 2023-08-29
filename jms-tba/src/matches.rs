use std::collections::HashMap;

use jms_base::kv;
use jms_core_lib::{scoring::scores::{SnapshotScore, GamepieceType, EndgameType, Link, MatchScoreSnapshot}, models::{self, PlayoffModeType, MatchType}, db::{Table, Singleton}};
use log::error;

use crate::{teams::TBATeam, client::TBAClient};

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
  pub score_breakdown: Option<TBA2023ScoreBreakdownFull>,
  pub time_str: Option<String>,
  pub time_utc: Option<String>
}

pub struct TBAMatchUpdate {}

impl TBAMatchUpdate {
  pub async fn issue(kv: &kv::KVConnection) -> anyhow::Result<()> {
    let playoff_type = models::PlayoffMode::get(kv)?;
    let matches = models::Match::all(kv)?;
    let scores = models::CommittedMatchScores::all_map(kv)?;

    for m in matches {
      if m.match_type != MatchType::Test {
        let latest_score = scores.get(&m.id).and_then(|cms| cms.scores.last()).map(|x| Into::<MatchScoreSnapshot>::into(x.clone()));
        
        // Try to convert our format into TBA's format
        let (comp_level, set, match_n) = match (&playoff_type.mode, m.match_type, m.round, m.set_number, m.match_number) {
          (_, MatchType::Qualification, _, s, _) => ( Some(TBAMatchLevel("qm")), 1, s ),
          (_, MatchType::Final, _, s, m) => ( Some(TBAMatchLevel("f")), s, m ),

          (PlayoffModeType::Bracket, MatchType::Playoff, r, s, m) => {
            match (r, s) {
              (1, x) => ( Some(TBAMatchLevel("qf")), x, m ),
              (2, x) => ( Some(TBAMatchLevel("sf")), x, m ),
              _ => ( None, 0, 0 )
            }
          },
          (PlayoffModeType::DoubleBracket, MatchType::Playoff, r, s, m) => {
            let tba_set = match (r, s) {
              (1, x) => x,
              (2, x) => 4 + x,
              (3, x) => 8 + x,
              (4, x) => 10 + x,
              (5, x) => 12 + x,
              (_, _) => 0
            };
            ( Some(TBAMatchLevel("sf")), tba_set, m )
          },
          _ => (None, 0, 0)
        };

        let mut tba_matches = vec![];

        // Fill TBA's match type
        match (comp_level, set, match_n) {
          ( Some(comp_level), set_number, match_number ) if set_number != 0 && match_number != 0 => {
            let red_teams = m.red_teams.iter().filter_map(|x| x.map(|t| TBATeam::from(t))).collect();
            let blue_teams = m.blue_teams.iter().filter_map(|x| x.map(|t| TBATeam::from(t))).collect();

            let tba_match = TBAMatch {
              comp_level, set_number, match_number,
              score_breakdown: latest_score.clone().map(Into::into),
              alliances: TBAMatchAlliances {
                red: TBAMatchAlliance { teams: red_teams, score: latest_score.as_ref().map(|x| x.red.derived.total_score as isize) },
                blue: TBAMatchAlliance { teams: blue_teams, score: latest_score.as_ref().map(|x| x.blue.derived.total_score as isize) }
              },
              time_str: Some(m.start_time.format("%l:%M %p").to_string()),
              time_utc: Some(chrono::DateTime::<chrono::Utc>::from(m.start_time).format("%+").to_string())
            };

            tba_matches.push(tba_match);
          },
          _ => error!("Could not convert match to TBA format: {}", m.id)
        }

        // Push to TBA
        if tba_matches.len() > 0 {
          TBAClient::post("matches", "update", &tba_matches, kv).await?;
        }
      }
    }

    Ok(())
  }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TBA2023ScoreBreakdownFull {
  blue: TBA2023ScoreBreakdown,
  red: TBA2023ScoreBreakdown,
}

impl From<MatchScoreSnapshot> for TBA2023ScoreBreakdownFull {
  fn from(score: MatchScoreSnapshot) -> Self {
    Self {
      red: score.red.into(),
      blue: score.blue.into()
    }
  }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct TBA2023ScoreBreakdown {
  // Thanks Cheesy-Arena :)
  mobilityRobot1: &'static str,
  mobilityRobot2: &'static str,
  mobilityRobot3: &'static str,
  autoMobilityPoints: isize,
  autoChargeStationRobot1: &'static str,
  autoChargeStationRobot2: &'static str,
  autoChargeStationRobot3: &'static str,
  autoBridgeState: &'static str,
  autoCommunity: HashMap<&'static str, Vec<&'static str>>,
  autoGamePieceCount: isize,
  autoGamePiecePoints: isize,
  autoPoints: isize,
  teleopCommunity: HashMap<&'static str, Vec<&'static str>>,
  teleopGamePieceCount: isize,
  teleopGamePiecePoints: isize,
  links: Vec<TBALink>,
  linkPoints: isize,
  extraGamePieceCount: isize,
  endGameChargeStationRobot1: &'static str,
  endGameChargeStationRobot2: &'static str,
  endGameChargeStationRobot3: &'static str,
  endGameBridgeState: &'static str,
  teleopPoints: isize,
  coopertitionCriteriaMet: bool,
  sustainabilityBonusAchieved: bool,
  activationBonusAchieved: bool,
  foulCount: isize,
  techFoulCount: isize,
  foulPoints: isize,
  totalPoints: isize,
  rp: isize,
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TBALink {
  nodes: [usize; 3],
  row: &'static str
}

impl From<Link> for TBALink {
  fn from(value: Link) -> Self {
    Self {
      nodes: value.nodes,
      row: match value.row {
        0 => "Bottom",
        1 => "Mid",
        _ => "Top"
      }
    }
  }
}

impl TBA2023ScoreBreakdown {
  pub fn yes_no(b: Option<&bool>) -> &'static str {
    if b == Some(&true) { "Yes" }
    else { "No" }
  }

  pub fn endgame_map(egt: Option<&EndgameType>) -> &'static str {
    match egt {
      Some(EndgameType::Docked) => "Docked",
      Some(EndgameType::Parked) => "Parked",
      _ => "None"
    }
  }

  pub fn convert_community(community: Vec<Vec<GamepieceType>>) -> HashMap<&'static str, Vec<&'static str>> {
    let mut map = HashMap::new();

    for (row_i, row) in community.into_iter().enumerate() {
      let v = row.into_iter().map(|gpt| match gpt {
        GamepieceType::None => "None",
        GamepieceType::Cone => "Cube",
        GamepieceType::Cube => "Cone",
      }).collect();

      map.insert(match row_i {
        0 => "B",
        1 => "M",
        _ => "T"
      }, v);
    }

    map
  }
}

impl From<SnapshotScore> for TBA2023ScoreBreakdown {
  fn from(score: SnapshotScore) -> Self {
    let live = score.live;
    let derived = score.derived;

    let mut teleop_community = live.community.auto.clone();
    for (i, row) in live.community.teleop.iter().enumerate() {
      for (j, col) in row.iter().enumerate() {
        if *col != GamepieceType::None {
          teleop_community[i][j] = *col;
        }
      }
    }

    Self {
      mobilityRobot1: Self::yes_no(live.mobility.get(0)),
      mobilityRobot2: Self::yes_no(live.mobility.get(1)),
      mobilityRobot3: Self::yes_no(live.mobility.get(2)),
      autoMobilityPoints: derived.mobility_points,
      autoChargeStationRobot1: if live.auto_docked { "Docked" } else { "None" },    // Our scores don't distinguish which robot docked.
      autoChargeStationRobot2: "None",
      autoChargeStationRobot3: "None",
      autoBridgeState: if live.charge_station_level.auto { "Level" } else { "NotLevel" },
      autoGamePieceCount: live.community.auto.iter().flat_map(|x| x).filter(|gp| *gp != &GamepieceType::None).count() as isize,
      autoCommunity: Self::convert_community(live.community.auto),
      autoGamePiecePoints: derived.community_points.auto,
      autoPoints: derived.mode_score.auto,
      teleopGamePieceCount: live.community.teleop.iter().flat_map(|x| x).filter(|gp| *gp != &GamepieceType::None).count() as isize,
      teleopCommunity: Self::convert_community(teleop_community),
      teleopGamePiecePoints: derived.community_points.teleop,
      links: derived.links.into_iter().map(|x| x.into()).collect(),
      linkPoints: derived.link_points,
      extraGamePieceCount: 0,   // We don't support supercharged nodes
      endGameChargeStationRobot1: Self::endgame_map(live.endgame.get(0)),
      endGameChargeStationRobot2: Self::endgame_map(live.endgame.get(1)),
      endGameChargeStationRobot3: Self::endgame_map(live.endgame.get(2)),
      endGameBridgeState: if live.charge_station_level.teleop { "Level" } else { "NotLevel" },
      teleopPoints: derived.mode_score.teleop,
      coopertitionCriteriaMet: derived.meets_coopertition,
      sustainabilityBonusAchieved: derived.sustainability_rp,
      activationBonusAchieved: derived.activation_rp,
      foulCount: live.penalties.fouls as isize,
      techFoulCount: live.penalties.tech_fouls as isize,
      foulPoints: live.penalties.fouls as isize * 5,
      totalPoints: live.penalties.tech_fouls as isize * 12,
      rp: derived.total_rp as isize,
    }
  }
}