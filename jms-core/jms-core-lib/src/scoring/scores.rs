use std::ops::Add;

use chrono::{Duration, Local};
use rand::{rngs::ThreadRng, Rng};

use crate::{db::{DBDuration, Singleton}, models::Alliance};

pub fn saturating_offset(base: usize, delta: isize) -> usize {
  if delta < 0 {
    base.checked_sub(delta.wrapping_abs() as usize).unwrap_or(0)
  } else {
    base.checked_add(delta as usize).unwrap_or(0)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ModeScore<T : PartialEq + Eq> {
  pub auto: T,
  pub teleop: T,
}

impl<T: PartialEq + Eq> ModeScore<T>
where
  T: Add + Copy,
{
  pub fn total(&self) -> T::Output {
    self.auto + self.teleop
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum EndgameType {
  None,
  Parked,
  Stage(usize)
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Penalties {
  pub fouls: usize,
  pub tech_fouls: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq, Eq)]
pub enum WinStatus {
  WIN,
  LOSS,
  TIE
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ScoringConfig {
  pub park_points: usize,
  pub onstage_points: usize,
  pub spotlight_points: usize,
  pub harmony_threshold: usize,
  pub harmony_points: usize,
  pub trap_points: usize,
  pub amp_auto_points: usize,
  pub amp_teleop_points: usize,
  pub speaker_auto_points: usize,
  pub speaker_amped_points: usize,
  pub speaker_unamped_points: usize,
  pub foul_points: usize,
  pub tech_foul_points: usize,
  pub leave_points: usize,
  pub melody_threshold_coop: usize,
  pub melody_threshold: usize,
  pub ensemble_points_threshold: usize,
  pub ensemble_stage_count_threshold: usize,
}

impl Default for ScoringConfig {
  fn default() -> Self {
    Self {
      park_points: 1,
      onstage_points: 3,
      spotlight_points: 1,
      harmony_threshold: 2,
      harmony_points: 2,
      trap_points: 5,
      amp_auto_points: 2,
      amp_teleop_points: 1,
      speaker_auto_points: 5,
      speaker_amped_points: 5,
      speaker_unamped_points: 2,
      foul_points: 2,
      tech_foul_points: 5,
      leave_points: 2,
      melody_threshold_coop: 15,
      melody_threshold: 18,
      ensemble_points_threshold: 10,
      ensemble_stage_count_threshold: 2,
    }
  }
}

impl Singleton for ScoringConfig {
  const KEY: &'static str = "score:config";
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct LiveNotes {
  pub banked: usize,
  pub amp: ModeScore<usize>,
  pub speaker_auto: usize,
  pub speaker_amped: usize,
  pub speaker_unamped: usize,
  pub amp_time: Option<chrono::DateTime<chrono::Local>>
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DerivedNotes {
  pub amp_points: ModeScore<isize>,
  pub speaker_auto_points: isize,
  pub speaker_amped_points: isize,
  pub speaker_unamped_points: isize,
  pub amplified_remaining: Option<DBDuration>,
  pub total_count: usize,
  pub total_points: isize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct LiveScore {
  pub leave: Vec<bool>,
  pub notes: LiveNotes,
  pub coop: bool,
  pub microphones: Vec<bool>,
  pub traps: Vec<bool>,
  pub endgame: Vec<EndgameType>,
  pub penalties: Penalties,
  pub coop_adjust: bool,
  pub melody_adjust: bool,
  pub ensemble_adjust: bool,
  pub adjustment: isize,
}

impl Default for LiveScore {
  fn default() -> Self {
    Self::new(3)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DerivedScore {
  pub leave_points: isize,
  pub notes: DerivedNotes,
  pub endgame_points: isize,
  pub endgame_park_points: isize,
  pub endgame_onstage_points: isize,
  pub endgame_harmony_points: isize,
  pub endgame_spotlit_bonus_points: isize,
  pub endgame_trap_points: isize,
  pub coopertition_met: bool,
  pub melody_threshold: usize,
  pub melody_rp: bool,
  pub ensemble_rp: bool,

  pub amplified_remaining: Option<DBDuration>,

  pub mode_score: ModeScore<isize>,
  pub penalty_score: usize,
  pub foul_score: usize,
  pub tech_foul_score: usize,
  pub total_score: usize,
  pub total_bonus_rp: usize,
  pub win_rp: usize,
  pub total_rp: usize,
  pub win_status: WinStatus,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct MatchScore {
  pub red: LiveScore,
  pub blue: LiveScore,
}

impl MatchScore {
  pub fn new(red_teams: usize, blue_teams: usize) -> MatchScore {
    MatchScore {
      red: LiveScore::new(red_teams),
      blue: LiveScore::new(blue_teams),
    }
  }

  pub fn winner(&self, config: ScoringConfig) -> Option<Alliance> {
    let red_derived = self.red.derive(&self.blue, config);
    let blue_derived = self.blue.derive(&self.red, config);

    if red_derived.total_score == blue_derived.total_score {
      None
    } else if red_derived.total_score > blue_derived.total_score {
      Some(Alliance::Red)
    } else {
      Some(Alliance::Blue)
    }
  }

  pub fn derive(self, config: ScoringConfig) -> MatchScoreSnapshot {
    let derive_red = self.red.derive(&self.blue, config);
    let derive_blue = self.blue.derive(&self.red, config);

    MatchScoreSnapshot {
      red: SnapshotScore {
        live: self.red,
        derived: derive_red,
      },
      blue: SnapshotScore {
        live: self.blue,
        derived: derive_blue,
      },
    }
  }
}

impl Singleton for MatchScore {
  const KEY: &'static str = "score:live";
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SnapshotScore {
  pub live: LiveScore,
  pub derived: DerivedScore,
}

impl PartialEq for SnapshotScore {
  fn eq(&self, other: &Self) -> bool {
    self.live == other.live && self.derived == other.derived
  }
}

impl Eq for SnapshotScore { }

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct MatchScoreSnapshot {
  pub red: SnapshotScore,
  pub blue: SnapshotScore,
}

impl From<MatchScoreSnapshot> for MatchScore {
  fn from(snapshot: MatchScoreSnapshot) -> Self {
    Self {
      red: snapshot.red.live,
      blue: snapshot.blue.live,
    }
  }
}

// For updating from the frontend.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub enum ScoreUpdate {
  Leave {
    station: usize,
    crossed: bool
  },
  Coop,
  Amplify,
  Microphone {
    stage: usize,
    activated: bool
  },
  Trap {
    stage: usize,
    filled: bool
  },
  Notes {
    auto: bool,
    #[serde(default)]
    speaker: isize,
    #[serde(default)]
    amp: isize
  },
  Endgame {
    station: usize,
    endgame: EndgameType,
  },
  Penalty {
    #[serde(default)]
    fouls: isize,
    #[serde(default)]
    tech_fouls: isize,
  },
}

#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct ScoreUpdateData {
  pub alliance: Alliance,
  pub update: ScoreUpdate,
}

impl LiveScore {
  pub fn new(num_teams: usize) -> Self {
    Self {
      leave: vec![false; num_teams],
      notes: LiveNotes {
        banked: 0,
        amp: ModeScore { auto: 0, teleop: 0 },
        speaker_auto: 0,
        speaker_amped: 0,
        speaker_unamped: 0,
        amp_time: None
      },
      coop: false,
      microphones: vec![false; 3],
      traps: vec![false; 3],
      endgame: vec![EndgameType::None; num_teams],
      penalties: Penalties {
        fouls: 0,
        tech_fouls: 0,
      },
      coop_adjust: false,
      melody_adjust: false,
      ensemble_adjust: false,
      adjustment: 0,
    }
  }

  pub fn partial_derive(&self, other_alliance: &LiveScore, config: ScoringConfig) -> DerivedScore {
    let melody_threshold = match (self.coop && other_alliance.coop) || self.coop_adjust {
      true => config.melody_threshold_coop,
      false => config.melody_threshold,
    };

    let mut endgame_park_points = 0isize;
    let mut endgame_onstage_points = 0isize;
    let mut endgame_spotlit_bonus_points = 0isize;
    let mut endgame_harmony_points = 0isize;
    let mut endgame_trap_points = 0isize;

    let mut stage_counts = [0; 3];

    // PARK / ONSTAGE Points
    for team_endgame in self.endgame.iter() {
      match team_endgame {
        EndgameType::None => {},
        EndgameType::Parked => {
          endgame_park_points += config.park_points as isize;
        },
        EndgameType::Stage(stage) => {
          let spotlit = self.microphones[*stage];
          stage_counts[*stage] += 1;
          endgame_onstage_points += config.onstage_points as isize;
          if spotlit {
            endgame_spotlit_bonus_points += config.spotlight_points as isize;
          }
        },
      }
    }

    // HARMONY points
    endgame_harmony_points += stage_counts.iter().filter(|&x| *x >= config.harmony_threshold).map(|x| *x - 1).count() as isize * config.harmony_points as isize;

    // Trap Points
    endgame_trap_points += self.traps.iter().filter(|&x| *x).count() as isize * config.trap_points as isize;

    let endgame_points = endgame_park_points + endgame_harmony_points + endgame_onstage_points + endgame_trap_points + endgame_spotlit_bonus_points;

    let amplified_remaining = match self.notes.amp_time {
      Some(x) => {
        let elapsed = Local::now() - x;
        if elapsed >= Duration::seconds(10) {
          None
        } else {
          Some(Duration::seconds(10) - elapsed)
        }
      },
      None => None
    };

    let total_notes = self.notes.amp.total() + self.notes.speaker_auto + self.notes.speaker_amped + self.notes.speaker_unamped;

    let amp_points = ModeScore { auto: (self.notes.amp.auto * config.amp_auto_points) as isize, teleop: (self.notes.amp.teleop * config.amp_teleop_points) as isize };
    let speaker_auto_points = (self.notes.speaker_auto * config.speaker_auto_points) as isize;
    let speaker_amped_points = (self.notes.speaker_amped * config.speaker_amped_points) as isize;
    let speaker_unamped_points = (self.notes.speaker_unamped * config.speaker_unamped_points) as isize;

    let foul_score = other_alliance.penalties.fouls * config.foul_points;
    let tech_foul_score = other_alliance.penalties.tech_fouls * config.tech_foul_points;

    let mut d = DerivedScore {
      leave_points: self.leave.iter().map(|x| (*x as isize) * config.leave_points as isize).sum(),
      notes: DerivedNotes {
        total_points: amp_points.auto + amp_points.teleop + speaker_auto_points + speaker_amped_points + speaker_unamped_points,
        amp_points,
        speaker_auto_points,
        speaker_amped_points,
        speaker_unamped_points,
        amplified_remaining: amplified_remaining.map(DBDuration),
        total_count: total_notes,
      },
      coopertition_met: self.coop,
      melody_threshold,
      melody_rp: (total_notes >= melody_threshold) || self.melody_adjust,
      ensemble_rp: (endgame_points >= config.ensemble_points_threshold as isize && self.endgame.iter().filter(|&x| matches!(*x, EndgameType::Stage(_))).count() >= config.ensemble_stage_count_threshold) || self.ensemble_adjust,
      endgame_harmony_points,
      endgame_onstage_points,
      endgame_park_points,
      endgame_trap_points,
      endgame_spotlit_bonus_points,
      endgame_points,

      foul_score,
      tech_foul_score,
      penalty_score: foul_score + tech_foul_score,
      
      amplified_remaining: self.amplified_remaining(),

      mode_score: ModeScore { auto: 0, teleop: 0 },
      total_score: 0,
      total_bonus_rp: 0,
      win_rp: 0,
      total_rp: 0,
      win_status: WinStatus::TIE,
    };

    d.mode_score = ModeScore {
      auto: d.leave_points + d.notes.amp_points.auto + d.notes.speaker_auto_points,
      teleop: d.notes.amp_points.teleop + d.notes.speaker_amped_points + d.notes.speaker_unamped_points + d.endgame_points,
    };
    d.total_score = (d.mode_score.auto as isize + d.mode_score.teleop as isize + d.penalty_score as isize + self.adjustment).max(0) as usize;

    d
  }

  pub fn derive(&self, other_alliance: &LiveScore, config: ScoringConfig) -> DerivedScore {
    let mut d = self.partial_derive(other_alliance, config);
    let other_d = other_alliance.partial_derive(self, config);
    
    let win_status = match d.total_score.cmp(&other_d.total_score) {
      std::cmp::Ordering::Greater => WinStatus::WIN,
      std::cmp::Ordering::Less => WinStatus::LOSS,
      std::cmp::Ordering::Equal => WinStatus::TIE,
    };

    d.total_bonus_rp = (d.melody_rp as usize) + (d.ensemble_rp as usize);
    d.win_rp = match win_status {
      WinStatus::WIN => 2,
      WinStatus::LOSS => 0,
      WinStatus::TIE => 1,
      };
    d.win_status = win_status;
    d.total_rp = d.win_rp + d.total_bonus_rp;

    d
  }

  // TODO: This needs better error handling - currently all inputs are assumed to be correct
  pub fn update(&mut self, score_update: ScoreUpdate) {
    match score_update {
      ScoreUpdate::Leave { station, crossed } => {
        self.leave[station] = crossed;
      },
      ScoreUpdate::Coop => {
        if self.notes.banked > 0 {
          self.coop = true;
          self.notes.banked = saturating_offset(self.notes.banked, -1);
        }
      },
      ScoreUpdate::Amplify => {
        if self.notes.banked >= 2 {
          self.notes.amp_time = Some(Local::now());
          self.notes.banked = 0;
        }
      },
      ScoreUpdate::Microphone { stage, activated } => {
        self.microphones[stage] = activated;
      },
      ScoreUpdate::Trap { stage, filled } => {
        self.traps[stage] = filled;
      },
      ScoreUpdate::Notes { auto, speaker, amp } => {
        let amplified = self.amplified_remaining().is_some();

        if auto {
          self.notes.amp.auto = saturating_offset(self.notes.amp.auto, amp);
          self.notes.speaker_auto = saturating_offset(self.notes.speaker_auto, speaker);

          self.notes.banked = 2.min(saturating_offset(self.notes.banked, amp));
        } else if amplified {
          self.notes.amp.teleop = saturating_offset(self.notes.amp.teleop, amp);
          self.notes.speaker_amped = saturating_offset(self.notes.speaker_amped, speaker);

          self.notes.banked = 2.min(saturating_offset(self.notes.banked, amp));
        } else {
          self.notes.amp.teleop = saturating_offset(self.notes.amp.teleop, amp);
          self.notes.speaker_unamped = saturating_offset(self.notes.speaker_unamped, speaker);

          self.notes.banked = 2.min(saturating_offset(self.notes.banked, amp));
        }
      },
      ScoreUpdate::Endgame { station, endgame } => {
        self.endgame[station] = endgame;
      },
      ScoreUpdate::Penalty { fouls, tech_fouls } => {
        self.penalties.fouls = saturating_offset(self.penalties.fouls, fouls);
        self.penalties.tech_fouls = saturating_offset(self.penalties.tech_fouls, tech_fouls);
      },
    }
  }

  fn amplified_remaining(&self) -> Option<DBDuration> {
    match self.notes.amp_time {
      Some(x) => {
        let elapsed = Local::now() - x;
        let max = Duration::seconds(10);
        if elapsed <= max {
          return Some(DBDuration(max - elapsed));
        } else {
          return None
        }
      },
      _ => None
    }
  }

  pub fn randomise() -> Self {
    let mut rng = rand::thread_rng();

    let rand_endgame = |rng: &mut ThreadRng| {
      match rng.gen_range(0..=5) {
        0 => EndgameType::None,
        1 => EndgameType::Parked,
        x => EndgameType::Stage(x - 2)
      }
    };

    let leave: Vec<bool> = vec![ rng.gen(), rng.gen(), rng.gen() ];

    let notes = LiveNotes {
      banked: 0,
      amp: ModeScore { auto: rng.gen_range(0..=1), teleop: rng.gen_range(0..=7) },
      speaker_auto: rng.gen_range(0..=5),
      speaker_amped: rng.gen_range(0..=10),
      speaker_unamped: rng.gen_range(0..=20),
      amp_time: None,
    };

    Self {
      leave,
      notes,
      coop: rng.gen(),
      microphones: vec![rng.gen(), rng.gen(), rng.gen()],
      traps: vec![rng.gen(), rng.gen(), rng.gen()],
      endgame: vec![rand_endgame(&mut rng), rand_endgame(&mut rng), rand_endgame(&mut rng)],
      penalties: Penalties {
        fouls: rng.gen_range(0..=4),
        tech_fouls: rng.gen_range(0..=2)
      },
      coop_adjust: false,
      melody_adjust: false,
      ensemble_adjust: false,
      adjustment: 0,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hop_qm16() {
    let blue = LiveScore {
      leave: vec![false, true, true],
      notes: LiveNotes { banked: 0, amp: ModeScore { auto: 0, teleop: 4 }, speaker_auto: 5, speaker_amped: 2, speaker_unamped: 8, amp_time: None },
      coop: true,
      microphones: vec![false, false, false],
      traps: vec![false, true, false],
      endgame: vec![EndgameType::None, EndgameType::Stage(1), EndgameType::Stage(2)],
      penalties: Penalties { fouls: 0, tech_fouls: 2 },
      coop_adjust: false,
      melody_adjust: false,
      ensemble_adjust: false,
      adjustment: 0,
    };

    let red = LiveScore {
      leave: vec![true, false, true],
      notes: LiveNotes { banked: 0, amp: ModeScore { auto: 0, teleop: 7 }, speaker_auto: 6, speaker_amped: 10, speaker_unamped: 0, amp_time: None },
      coop: true,
      microphones: vec![false, false, false],
      traps: vec![false, false, false],
      endgame: vec![EndgameType::Parked, EndgameType::Stage(1), EndgameType::Parked],
      penalties: Penalties { fouls: 1, tech_fouls: 0 },
      coop_adjust: false,
      melody_adjust: false,
      ensemble_adjust: false,
      adjustment: 0,
    };

    let mut config = ScoringConfig::default();

    // This was at the FIRST Championship, so there's new limits
    config.melody_threshold = 25;
    config.melody_threshold_coop = 21;

    let derived_blue = blue.derive(&red, config);
    let derived_red = red.derive(&blue, config);

    assert_eq!(derived_blue.notes.total_points, 55);
    assert_eq!(derived_blue.leave_points, 4);
    assert_eq!(derived_blue.endgame_points, 11);

    assert_eq!(derived_red.notes.total_points, 87);
    assert_eq!(derived_red.leave_points, 4);
    assert_eq!(derived_red.endgame_points, 5);

    assert_eq!(derived_blue.total_score, 72);
    assert_eq!(derived_blue.total_rp, 1);

    // NOTE: The extra ensemble RP comes from G424, which in JMS is allocated after the scores are entered.
    assert_eq!(derived_red.total_score, 106);
    assert_eq!(derived_red.total_rp, 3);
  }
}
