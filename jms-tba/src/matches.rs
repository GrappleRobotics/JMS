use jms_base::kv;
use jms_core_lib::{db::{Singleton, Table}, models::{self, MatchType, PlayoffModeType}, scoring::scores::{EndgameType, MatchScoreSnapshot, ScoringConfig, SnapshotScore}};
use log::{error, info, warn};

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
  pub score_breakdown: Option<TBA2024ScoreBreakdownFull>,
  pub time_str: Option<String>,
  pub time_utc: Option<String>
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TBAGetMatch {
  pub key: String,
  pub comp_level: String,
  pub set_number: usize,
  pub match_number: usize,
}

pub struct TBAMatchUpdate {}

impl TBAMatchUpdate {
  pub async fn issue(kv: &kv::KVConnection) -> anyhow::Result<()> {
    let mut to_delete: Vec<TBAGetMatch> = match TBAClient::get("matches/simple", kv).await {
      Ok(v) => v,
      Err(_) => vec![]
    };

    let playoff_type = models::PlayoffMode::get(kv)?;
    let matches = models::Match::all(kv)?;
    let scores = models::CommittedMatchScores::all_map(kv)?;
    let score_config = ScoringConfig::get(kv)?;

    for m in matches {
      if m.match_type != MatchType::Test {
        let latest_score = scores.get(&m.id).and_then(|cms| cms.scores.last()).map(|x| x.clone().derive(score_config));
        
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

            let pos = to_delete.iter().position(|x| x.comp_level == tba_match.comp_level.0 && x.match_number == tba_match.match_number && x.set_number == tba_match.set_number);
            if let Some(pos) = pos {
              to_delete.remove(pos);
            }
            tba_matches.push(tba_match);
          },
          _ => error!("Could not convert match to TBA format: {}", m.id)
        }

        info!("Updating Matches...");
        if tba_matches.len() > 0 {
          match TBAClient::post("matches", "update", &tba_matches, kv).await {
            Ok(_) => {},
            Err(_) => {
              // Try again but without the breakdown
              warn!("Could not publish match update with score breakdown, trying again without the breakdown");

              for m in &mut tba_matches {
                m.score_breakdown = None;
              }

              TBAClient::post("matches", "update", &tba_matches, kv).await?;
            },
          }
        }
      }
    }

    // Push to TBA

    // TODO: These keys should be substringed to just the fragment
    if to_delete.len() > 0 {
      let to_del = to_delete.iter().filter_map(|x| x.key.split_once("_").map(|x| x.1.to_owned())).collect::<Vec<String>>();
      info!("Matches to Delete: {:?}", to_del);
      match TBAClient::post("matches", "delete", &to_del, kv).await {
        Ok(_) => (),
        Err(e) => error!("Could not delete matches: {}", e)
      }
    } else {
      info!("No matches to delete");
    }

    Ok(())
  }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TBA2024ScoreBreakdownFull {
  blue: TBA2024ScoreBreakdown,
  red: TBA2024ScoreBreakdown,
}

impl From<MatchScoreSnapshot> for TBA2024ScoreBreakdownFull {
  fn from(score: MatchScoreSnapshot) -> Self {
    Self {
      red: score.red.into(),
      blue: score.blue.into()
    }
  }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct TBA2024ScoreBreakdown {
  // Thanks Cheesy-Arena :)
  autoLineRobot1: &'static str,
  autoLineRobot2: &'static str,
  autoLineRobot3: &'static str,
  autoLeavePoints: isize,

  autoAmpNoteCount: isize,
  autoAmpNotePoints: isize,
  autoSpeakerNoteCount: isize,
  autoSpeakerNotePoints: isize,
  autoTotalNotePoints: isize,
  autoPoints: isize,

  teleopAmpNoteCount: isize,
  teleopAmpNotePoints: isize,
  teleopSpeakerNoteCount: isize,
  teleopSpeakerNotePoints: isize,
  teleopSpeakerNoteAmplifiedCount: isize,
  teleopSpeakerNoteAmplifiedPoints: isize,
  teleopTotalNotePoints: isize,

  endGameRobot1: &'static str,
  endGameRobot2: &'static str,
  endGameRobot3: &'static str,

  endGameParkPoints: isize,
  endGameOnStagePoints: isize,
  endGameHarmonyPoints: isize,
  micStageLeft: bool,
  micCenterStage: bool,
  micStageRight: bool,

  endGameSpotLightBonusPoints: isize,

  trapStageLeft: bool,
  trapCenterStage: bool,
  trapStageRight: bool,

  endGameNoteInTrapPoints: isize,
  endGameTotalStagePoints: isize,
  teleopPoints: isize,

  coopertitionCriteriaMet: bool,
  melodyBonusAchieved: bool,
  ensembleBonusAchieved: bool,

  foulCount: isize,
  techFoulCount: isize,
  g424Penalty: bool,
  foulPoints: isize,
  totalPoints: isize,
  rp: isize,
}

impl TBA2024ScoreBreakdown {
  pub fn yes_no(b: Option<&bool>) -> &'static str {
    if b == Some(&true) { "Yes" }
    else { "No" }
  }

  pub fn endgame_map(egt: Option<&EndgameType>) -> &'static str {
    match egt {
      Some(EndgameType::None) => "None",
      Some(EndgameType::Parked) => "Parked",
      Some(EndgameType::Stage(0)) => "CenterStage",
      Some(EndgameType::Stage(1)) => "StageLeft",
      Some(EndgameType::Stage(2)) => "StageRight",
      _ => "None"
    }
  }
}

impl From<SnapshotScore> for TBA2024ScoreBreakdown {
  fn from(score: SnapshotScore) -> Self {
    let live = score.live;
    let derived = score.derived;

    Self {
      autoLineRobot1: Self::yes_no(live.leave.get(0)),
      autoLineRobot2: Self::yes_no(live.leave.get(1)),
      autoLineRobot3: Self::yes_no(live.leave.get(2)),
      autoLeavePoints: derived.leave_points,
      autoAmpNoteCount: live.notes.amp.auto as isize,
      autoAmpNotePoints: derived.notes.amp_points.auto,
      autoSpeakerNoteCount: live.notes.speaker_auto as isize,
      autoSpeakerNotePoints: derived.notes.speaker_auto_points,
      autoTotalNotePoints: derived.notes.amp_points.auto + derived.notes.speaker_auto_points,
      autoPoints: derived.mode_score.auto,
      teleopAmpNoteCount: live.notes.amp.teleop as isize,
      teleopAmpNotePoints: derived.notes.amp_points.teleop,
      teleopSpeakerNoteCount: live.notes.speaker_unamped as isize,
      teleopSpeakerNotePoints: derived.notes.speaker_unamped_points,
      teleopSpeakerNoteAmplifiedCount: live.notes.speaker_amped as isize,
      teleopSpeakerNoteAmplifiedPoints: derived.notes.speaker_amped_points,
      teleopTotalNotePoints: derived.notes.amp_points.teleop + derived.notes.speaker_unamped_points + derived.notes.speaker_amped_points,
      endGameRobot1: Self::endgame_map(live.endgame.get(0)),
      endGameRobot2: Self::endgame_map(live.endgame.get(1)),
      endGameRobot3: Self::endgame_map(live.endgame.get(2)),
      endGameParkPoints: derived.endgame_park_points,
      endGameOnStagePoints: derived.endgame_onstage_points,
      endGameHarmonyPoints: derived.endgame_harmony_points,
      micStageLeft: live.microphones.get(1) == Some(&true),
      micCenterStage: live.microphones.get(0) == Some(&true),
      micStageRight: live.microphones.get(2) == Some(&true),
      endGameSpotLightBonusPoints: derived.endgame_spotlit_bonus_points,
      trapStageLeft: live.traps.get(1) == Some(&true),
      trapCenterStage: live.traps.get(0) == Some(&true),
      trapStageRight: live.traps.get(2) == Some(&true),
      endGameNoteInTrapPoints: derived.endgame_trap_points,
      endGameTotalStagePoints: derived.endgame_points,
      teleopPoints: derived.mode_score.teleop,
      coopertitionCriteriaMet: derived.coopertition_met,
      melodyBonusAchieved: derived.melody_rp,
      ensembleBonusAchieved: derived.ensemble_rp,
      foulCount: live.penalties.fouls as isize,
      techFoulCount: live.penalties.tech_fouls as isize,
      g424Penalty: false,
      foulPoints: derived.penalty_score as isize,
      totalPoints: derived.total_score as isize,
      rp: derived.total_rp as isize,
    }
  }
}