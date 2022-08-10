use std::ops::Add;

use rand::{Rng, prelude::ThreadRng};

use crate::{models::Alliance, utils::saturating_offset};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ModeScore<T> {
  pub auto: T,
  pub teleop: T,
}

impl<T> ModeScore<T>
where
  T: Add + Copy,
{
  pub fn total(&self) -> T::Output {
    self.auto + self.teleop
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq, Eq)]
pub enum EndgamePointType {
  None,
  Low,
  Mid,
  High,
  Traversal
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CargoCounts {
  pub upper: [usize; 4],
  pub lower: [usize; 4],
}

impl CargoCounts {
  pub fn total_cargo(&self) -> usize {
    return self.upper.iter().sum::<usize>() + self.lower.iter().sum::<usize>();
  }
}

impl Add for CargoCounts {
  type Output = CargoCounts;

  fn add(self, rhs: Self) -> Self::Output {
    let mut upper = self.upper.clone();
    let mut lower = self.lower.clone();

    for idx in 0..4 {
      upper[idx] += rhs.upper[idx];
      lower[idx] += rhs.lower[idx];
    }

    return CargoCounts { upper, lower };
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct LiveScore {
  pub taxi: Vec<bool>,
  pub cargo: ModeScore<CargoCounts>,
  pub endgame: Vec<EndgamePointType>,
  pub penalties: Penalties,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DerivedScore {
  pub taxi_points: isize,
  pub cargo_points: ModeScore<isize>,
  pub endgame_points: isize,
  pub cargo_rp: bool,
  pub hangar_rp: bool,
  pub quintet: bool,
  pub mode_score: ModeScore<isize>,
  pub penalty_score: usize,
  pub total_score: usize,
  pub total_bonus_rp: usize,
  pub win_rp: usize,
  pub total_rp: usize,
  pub win_status: WinStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SnapshotScore {
  pub live: LiveScore,
  pub derived: DerivedScore,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
  Taxi {
    station: usize,
    crossed: bool,
  },
  Cargo {
    auto: bool,
    #[serde(default)]
    upper: [isize; 4],
    #[serde(default)]
    lower: [isize; 4],
  },
  Endgame {
    station: usize,
    endgame: EndgamePointType,
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

// Game Manual 4.4.5
impl LiveScore {
  pub fn new(num_teams: usize) -> Self {
    Self {
      taxi: vec![false; num_teams],
      cargo: ModeScore {
        auto: CargoCounts {
          upper: [0; 4],
          lower: [0; 4],
        },
        teleop: CargoCounts {
          upper: [0; 4],
          lower: [0; 4],
        },
      },
      endgame: vec![EndgamePointType::None; num_teams],
      penalties: Penalties {
        fouls: 0,
        tech_fouls: 0,
      },
    }
  }

  pub fn derive(&self, other_alliance: &LiveScore) -> DerivedScore {
    let penalty_points = other_alliance.penalty_points_other_alliance();
    let mode_score = self.mode_score();

    let total_score = mode_score.total() as usize + penalty_points;
    let other_total_score = other_alliance.mode_score().total() as usize + self.penalty_points_other_alliance();

    let (win_status, win_rp) = match (total_score, other_total_score) {
      (a, b) if a > b => ( WinStatus::WIN, 2 ),
      (a, b) if a < b => ( WinStatus::LOSS, 0 ),
      _ => ( WinStatus::TIE, 1 )
    };

    DerivedScore {
      taxi_points: self.taxi_points(),
      cargo_points: self.cargo_points(),
      endgame_points: self.endgame_points(),
      cargo_rp: self.cargo_rp(),
      hangar_rp: self.hangar_rp(),
      penalty_score: penalty_points,
      quintet: self.quintet(),
      total_score,
      mode_score,
      total_bonus_rp: self.total_bonus_rp(),
      win_rp,
      total_rp: self.total_bonus_rp() + win_rp,
      win_status,
    }
  }

  // TODO: This needs better error handling - currently all inputs are assumed to be correct
  pub fn update(&mut self, score_update: ScoreUpdate) {
    match score_update {
      ScoreUpdate::Taxi { station, crossed } => {
        self.taxi[station] = crossed;
      }
      ScoreUpdate::Cargo {
        auto,
        upper,
        lower
      } => {
        let pc = if auto {
          &mut self.cargo.auto
        } else {
          &mut self.cargo.teleop
        };

        for idx in 0..4 {
          pc.lower[idx] = saturating_offset(pc.lower[idx], lower[idx]);
          pc.upper[idx] = saturating_offset(pc.upper[idx], upper[idx]);
        }
      }
      ScoreUpdate::Endgame { station, endgame } => {
        self.endgame[station] = endgame;
      }
      ScoreUpdate::Penalty { fouls, tech_fouls } => {
        self.penalties.fouls = saturating_offset(self.penalties.fouls, fouls);
        self.penalties.tech_fouls = saturating_offset(self.penalties.tech_fouls, tech_fouls);
      }
    }
  }

  fn taxi_points(&self) -> isize {
    self.taxi.iter().map(|&x| (x as isize) * 2).sum()
  }

  fn cargo_points(&self) -> ModeScore<isize> {
    let auto = &self.cargo.auto;
    let teleop = &self.cargo.teleop;
    ModeScore {
      auto: (auto.lower.iter().sum::<usize>() * 2 + auto.upper.iter().sum::<usize>() * 4) as isize,
      teleop: (teleop.lower.iter().sum::<usize>() * 1 + teleop.upper.iter().sum::<usize>() * 2) as isize,
    }
  }

  // fn total_cells(&self) -> usize {
  //   let auto = &self.power_cells.auto;
  //   let teleop = &self.power_cells.teleop;
  //   auto.inner + auto.outer + auto.bottom + teleop.inner + teleop.outer + teleop.bottom
  // }

  fn endgame_points(&self) -> isize {
    self.endgame.iter().map(|x| match x {
      EndgamePointType::None => 0,
      EndgamePointType::Low => 4,
      EndgamePointType::Mid => 6,
      EndgamePointType::High => 10,
      EndgamePointType::Traversal => 15,
    }).sum()
  }

  fn cargo_rp(&self) -> bool {
    self.cargo.total().total_cargo() > match self.quintet() {
      true => 18,
      false => 20
    }
  }

  fn quintet(&self) -> bool {
    self.cargo.auto.total_cargo() >= 5
  }

  fn hangar_rp(&self) -> bool {
    self.endgame_points() >= 16
  }

  fn penalty_points_other_alliance(&self) -> usize {
    self.penalties.fouls * 4 + self.penalties.tech_fouls * 8
  }

  fn mode_score(&self) -> ModeScore<isize> {
    let cargo_points = self.cargo_points();
    ModeScore {
      auto: self.taxi_points() + cargo_points.auto,
      teleop: cargo_points.teleop + self.endgame_points(),
    }
  }

  fn total_bonus_rp(&self) -> usize {
    self.cargo_rp() as usize + self.hangar_rp() as usize
  }

  pub fn randomise() -> Self {
    let mut rng = rand::thread_rng();

    let rand_endgame = |rng: &mut ThreadRng| {
      match rng.gen_range(0..=4) {
        0 => EndgamePointType::None,
        1 => EndgamePointType::Low,
        2 => EndgamePointType::Mid,
        3 => EndgamePointType::High,
        _ => EndgamePointType::Traversal
      }
    };

    Self {
      taxi: vec![ rng.gen(), rng.gen(), rng.gen() ],
      cargo: ModeScore {
        auto: CargoCounts {
          lower: [rng.gen_range(0..=1), rng.gen_range(0..=1), rng.gen_range(0..=1), rng.gen_range(0..=1)],
          upper: [rng.gen_range(0..=1), rng.gen_range(0..=1), rng.gen_range(0..=1), rng.gen_range(0..=1)],
        },
        teleop: CargoCounts {
          lower: [rng.gen_range(0..=5), rng.gen_range(0..=5), rng.gen_range(0..=5), rng.gen_range(0..=5)],
          upper: [rng.gen_range(0..=5), rng.gen_range(0..=5), rng.gen_range(0..=5), rng.gen_range(0..=5)],
        }
      },
      endgame: vec![ rand_endgame(&mut rng), rand_endgame(&mut rng), rand_endgame(&mut rng) ],
      penalties: Penalties {
        fouls: rng.gen_range(0..=4),
        tech_fouls: rng.gen_range(0..=2)
      }
    }
  }
}
