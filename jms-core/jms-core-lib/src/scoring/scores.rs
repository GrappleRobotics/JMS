use std::{ops::Add, time::Instant};

use chrono::{DateTime, Duration, Local};
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
  pub adjustment: isize,
}

impl Default for LiveScore {
  fn default() -> Self {
    Self::new(3)
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DerivedScore {
  pub leave_points: isize,
  pub notes: DerivedNotes,
  pub endgame_points: isize,
  pub coopertition_met: bool,
  pub melody_threshold: usize,
  pub melody_rp: bool,
  pub ensemble_rp: bool,

  pub mode_score: ModeScore<isize>,
  pub penalty_score: usize,
  pub total_score: usize,
  pub total_bonus_rp: usize,
  pub win_rp: usize,
  pub total_rp: usize,
  pub win_status: WinStatus,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
// #[serde(into = "MatchScoreSnapshot", from = "MatchScoreSnapshot")]
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

  pub fn winner(&self) -> Option<Alliance> {
    let red_derived = self.red.derive(&self.blue);
    let blue_derived = self.blue.derive(&self.red);

    if red_derived.total_score == blue_derived.total_score {
      None
    } else if red_derived.total_score > blue_derived.total_score {
      Some(Alliance::Red)
    } else {
      Some(Alliance::Blue)
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
    self.live == other.live
  }
}

impl Eq for SnapshotScore { }

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct MatchScoreSnapshot {
  pub red: SnapshotScore,
  pub blue: SnapshotScore,
}

impl Into<MatchScoreSnapshot> for MatchScore {
  fn into(self) -> MatchScoreSnapshot {
    let derive_red = self.red.derive(&self.blue);
    let derive_blue = self.blue.derive(&self.red);

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
      adjustment: 0,
    }
  }

  pub fn partial_derive(&self, other_alliance: &LiveScore) -> DerivedScore {
    let melody_threshold = match self.coop && other_alliance.coop {
      true => 15,
      false => 18,
    };
    
    // TODO: Calculate End-game
    // let endgame_points = self.endgame.iter().map(|x| match (*x, &self.microphones[..]) {
    //   (EndgameType::None, _) => 0,
    //   (EndgameType::Parked, _) => 1,
    //   (EndgameType::StageLeft, &[true, _, _]) => 4,
    //   (EndgameType::StageLeft, &[false, _, _]) => 3,
    // });

    let endgame_points = 0;

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

    let total_notes = self.notes.amp.total() + self.notes.speaker_amped + self.notes.speaker_unamped;

    let mut d = DerivedScore {
      leave_points: self.leave.iter().map(|x| (*x as isize) * 2).sum(),
      notes: DerivedNotes {
        amp_points: ModeScore { auto: (self.notes.amp.auto * 2) as isize, teleop: (self.notes.amp.teleop * 1) as isize },
        speaker_auto_points: (self.notes.speaker_auto * 5) as isize,
        speaker_amped_points: (self.notes.speaker_amped * 5) as isize,
        speaker_unamped_points: (self.notes.speaker_unamped * 2) as isize,
        amplified_remaining: amplified_remaining.map(DBDuration),
        total_count: total_notes
      },
      coopertition_met: self.coop,
      melody_threshold,
      melody_rp: total_notes >= melody_threshold,
      ensemble_rp: endgame_points >= 10 && self.endgame.iter().filter(|&x| matches!(*x, EndgameType::Stage(_))).count() >= 2,
      endgame_points,

      penalty_score: other_alliance.penalties.fouls * 2 + other_alliance.penalties.tech_fouls * 5,
      
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

  pub fn derive(&self, other_alliance: &LiveScore) -> DerivedScore {
    let mut d = self.partial_derive(other_alliance);
    let other_d = other_alliance.partial_derive(self);
    
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
        self.coop = true;
      },
      ScoreUpdate::Amplify => {
        self.notes.amp_time = Some(Local::now());
      },
      ScoreUpdate::Microphone { stage, activated } => {
        self.microphones[stage] = activated;
      },
      ScoreUpdate::Trap { stage, filled } => {
        self.traps[stage] = filled;
      },
      ScoreUpdate::Notes { auto, speaker, amp } => {
        let amplified = match self.notes.amp_time {
          Some(x) if (Local::now() - x) <= Duration::seconds(10) => true,
          _ => false
        };

        if auto {
          self.notes.amp.auto = saturating_offset(self.notes.amp.auto, amp);
          self.notes.speaker_auto = saturating_offset(self.notes.speaker_auto, speaker);
        } else if amplified {
          self.notes.amp.auto = saturating_offset(self.notes.amp.teleop, amp);
          self.notes.speaker_amped = saturating_offset(self.notes.speaker_amped, speaker);
        } else {
          self.notes.amp.auto = saturating_offset(self.notes.amp.teleop, amp);
          self.notes.speaker_unamped = saturating_offset(self.notes.speaker_unamped, speaker);
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

  pub fn randomise() -> Self {
    // let mut rng = rand::thread_rng();

    // let rand_endgame = |rng: &mut ThreadRng| {
    //   match rng.gen_range(0..=2) {
    //     0 => EndgameType::None,
    //     1 => EndgameType::Parked,
    //     _ => EndgameType::Docked
    //   }
    // };

    // let mut community = ModeScore {
    //   auto: vec![vec![GamepieceType::None; 9], vec![GamepieceType::None; 9], vec![GamepieceType::None; 9]],
    //   teleop: vec![vec![GamepieceType::None; 9], vec![GamepieceType::None; 9], vec![GamepieceType::None; 9]],
    // };

    // let auto_pieces = rng.gen_range(0..=2);
    // let teleop_pieces = rng.gen_range(0..=10);

    // for i in 0..(auto_pieces + teleop_pieces) {
    //   let row = rng.gen_range(0..3);
    //   let col = rng.gen_range(0..9);

    //   let allowed = match row {
    //     0 => vec![GamepieceType::Cube, GamepieceType::Cone],
    //     _ => match col {
    //       x if x % 3 == 1 => vec![GamepieceType::Cube],
    //       _ => vec![GamepieceType::Cone]
    //     }
    //   };

    //   let selected = allowed[rng.gen_range(0..allowed.len())];

    //   if i < auto_pieces {
    //     community.auto[row][col] = selected;
    //   } else {
    //     community.teleop[row][col] = selected;
    //   }
    // }

    // Self {
    //   mobility: vec![rng.gen(), rng.gen(), rng.gen()],
    //   community,
    //   auto_docked: rng.gen(),
    //   charge_station_level: ModeScore { auto: rng.gen(), teleop: rng.gen() },
    //   endgame: vec![rand_endgame(&mut rng), rand_endgame(&mut rng), rand_endgame(&mut rng)],
    //   penalties: Penalties {
    //     fouls: rng.gen_range(0..=4),
    //     tech_fouls: rng.gen_range(0..=2)
    //   },
    //   sustainability_adjust: false,
    //   activation_adjust: false,
    //   adjustment: 0
    // }
    Self::new(3)
  }
}
