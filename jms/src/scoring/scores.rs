use std::ops::Add;

use serde::ser::SerializeStruct;

use crate::{arena::station::Alliance, utils::saturating_offset};

// NOTE: WARP 2021 has some rule modifications.
// - The Control Panel is not included in our tournament
// - Stage capacities are absolute and do not depend on match state. 
//    - Stage 1: 9 Cells
//    - Stage 2: 9+15 = 24 Cells
//    - Stage 3: 9+15+15 = 39 Cells

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModeScore<T> {
  pub auto: T,
  pub teleop: T
}

impl<T> ModeScore<T>
  where T: Add + Copy
{
  pub fn total(&self) -> T::Output {
    self.auto + self.teleop
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum EndgamePointType {
  None,
  Hang,
  Park
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PowerCellCounts {
  pub inner: usize,
  pub outer: usize,
  pub bottom: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LiveScore {
  pub initiation_line_crossed: Vec<bool>,
  pub power_cells: ModeScore<PowerCellCounts>,
  // TODO: Control Panel (if we are about it)
  pub endgame: Vec<EndgamePointType>,
  pub rung_level: bool,
  // TODO: Deductions / Fouls / Cards
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DerivedScore {
  pub initiation_points: isize,
  pub cell_points: ModeScore<isize>,
  pub endgame_points: isize,
  pub shield_gen_rp: bool,
  pub stage3_rp: bool,
  pub stage: usize,
  pub total_score: ModeScore<isize>,
  pub total_bonus_rp: usize
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(into="MatchScoreSnapshot", from="MatchScoreSnapshot")]
pub struct MatchScore {
  pub red: LiveScore,
  pub blue: LiveScore
}

impl MatchScore {
  pub fn new(red_teams: usize, blue_teams: usize) -> MatchScore {
    MatchScore {
      red: LiveScore::new(red_teams),
      blue: LiveScore::new(blue_teams)
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotScore {
  pub live: LiveScore,
  pub derived: DerivedScore
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchScoreSnapshot {
  pub red: SnapshotScore,
  pub blue: SnapshotScore
}

impl Into<MatchScoreSnapshot> for MatchScore {
  fn into(self) -> MatchScoreSnapshot {
    let derive_red = self.red.derive();
    let derive_blue = self.blue.derive();
    
    MatchScoreSnapshot {
      red: SnapshotScore { live: self.red, derived: derive_red },
      blue: SnapshotScore { live: self.blue, derived: derive_blue }
    }
  }
}

impl From<MatchScoreSnapshot> for MatchScore {
  fn from(snapshot: MatchScoreSnapshot) -> Self {
    Self {
      red: snapshot.red.live,
      blue: snapshot.blue.live
    }
  }
}

// // Special serialize to auto-derive the scores for JSON transport
// impl serde::Serialize for MatchScore {
//   fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//   where
//     S: serde::Serializer
//   {
//     #[derive(serde::Serialize)]
//     struct PairedScore {
//       live: LiveScore,
//       derived: DerivedScore
//     }

//     let red = PairedScore { live: self.red.clone(), derived: self.red.derive() };
//     let blue = PairedScore { live: self.blue.clone(), derived: self.blue.derive() };

//     let mut state = serializer.serialize_struct("MatchScore", 2)?;
//     state.serialize_field("red", &red)?;
//     state.serialize_field("blue", &blue)?;
//     state.end()
//   }
// }

// For updating from the frontend. 
#[derive(Debug, Clone, serde::Deserialize)]
pub enum ScoreUpdate {
  Initiation { station: usize, crossed: bool },
  PowerCell {
    auto: bool,
    #[serde(default)]
    inner: isize,
    #[serde(default)]
    outer: isize,
    #[serde(default)]
    bottom: isize,
  },
  Endgame { station: usize, endgame: EndgamePointType },
  RungLevel(bool)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ScoreUpdateData {
  pub alliance: Alliance,
  pub update: ScoreUpdate,
}

// Game Manual 4.4.5
impl LiveScore {
  pub fn new(num_teams: usize) -> Self {
    Self {
      initiation_line_crossed: vec![false; num_teams],
      power_cells: ModeScore {
        auto: PowerCellCounts { outer: 0, inner: 0, bottom: 0 },
        teleop: PowerCellCounts { outer: 0, inner: 0, bottom: 0 },
      },
      endgame: vec![EndgamePointType::None; num_teams],
      rung_level: false
    }
  }

  pub fn derive(&self) -> DerivedScore {
    DerivedScore {
      initiation_points: self.initiation_points(),
      cell_points: self.cell_points(),
      endgame_points: self.endgame_points(),
      shield_gen_rp: self.shield_gen_rp(),
      stage3_rp: self.stage3_rp(),
      stage: self.shield_gen_stage(),
      total_score: self.total_score(),
      total_bonus_rp: self.total_bonus_rp()
    }
  }

  // TODO: This needs better error handling - currently all inputs are assumed to be correct
  pub fn update(&mut self, score_update: ScoreUpdate) {
    match score_update {
      ScoreUpdate::Initiation { station, crossed } => {
        self.initiation_line_crossed[station] = crossed;
      },
      ScoreUpdate::PowerCell { auto, inner, outer, bottom } => {
        let pc = if auto { &mut self.power_cells.auto } else { &mut self.power_cells.teleop };
        pc.inner = saturating_offset(pc.inner, inner);
        pc.outer = saturating_offset(pc.outer, outer);
        pc.bottom = saturating_offset(pc.bottom, bottom);
      },
      ScoreUpdate::Endgame { station, endgame } => {
        self.endgame[station] = endgame;
      },
      ScoreUpdate::RungLevel(is_level) => {
        self.rung_level = is_level;
      },
    }
  }

  fn initiation_points(&self) -> isize {
    self.initiation_line_crossed.iter().map(|&x| (x as isize) * 5).sum()
  }

  fn cell_points(&self) -> ModeScore<isize> {
    let auto = &self.power_cells.auto;
    let teleop = &self.power_cells.teleop;
    ModeScore {
      auto: (auto.inner * 6 + auto.outer * 4 + auto.bottom * 2) as isize,
      teleop: (teleop.inner * 3 + teleop.outer * 2 + teleop.bottom * 1) as isize
    }
  }

  fn total_cells(&self) -> usize {
    let auto = &self.power_cells.auto;
    let teleop = &self.power_cells.teleop;
    auto.inner + auto.outer + auto.bottom + teleop.inner + teleop.outer + teleop.bottom
  }

  fn endgame_points(&self) -> isize {
    let n_hang = self.endgame.iter().filter(|&x| *x == EndgamePointType::Hang).count();
    let n_park = self.endgame.iter().filter(|&x| *x == EndgamePointType::Park).count();
    let has_level_points = self.rung_level && n_hang > 0;
    ((n_hang * 25 + n_park * 5) + (has_level_points as usize) * 15) as isize
  }

  fn shield_gen_rp(&self) -> bool {
    self.endgame_points() > 65
  }

  fn shield_gen_stage(&self) -> usize {
    let cell_count = self.total_cells();
    match cell_count {
      _ if cell_count >= 39 => 3,
      _ if cell_count >= 24 => 2,
      _ if cell_count >=  9 => 1,
      _ => 0
    }
  }

  fn stage3_rp(&self) -> bool {
    self.shield_gen_stage() == 3
  }

  fn total_score(&self) -> ModeScore<isize> {
    let cell_points = self.cell_points();
    ModeScore {
      auto: self.initiation_points() + cell_points.auto,
      teleop: cell_points.teleop + self.endgame_points()
    }
  }

  fn total_bonus_rp(&self) -> usize {
    self.shield_gen_stage() as usize + self.stage3_rp() as usize
  }
}