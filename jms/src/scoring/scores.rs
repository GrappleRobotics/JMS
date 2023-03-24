use std::ops::Add;

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

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum GamepieceType {
  None,
  Cone,
  Cube
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum EndgameType {
  None,
  Parked,
  Docked
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
  pub mobility: Vec<bool>,
  pub community: ModeScore<Vec<Vec<GamepieceType>>>,
  pub auto_docked: bool,
  pub charge_station_level: ModeScore<bool>,
  pub endgame: Vec<EndgameType>,
  pub penalties: Penalties,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DerivedScore {
  pub mobility_points: isize,
  pub link_points: isize,
  pub community_points: ModeScore<isize>,
  pub auto_docked_points: isize,
  pub endgame_points: isize,
  pub meets_coopertition: bool,
  pub sustainability_threshold: usize,
  pub sustainability_rp: bool,
  pub activation_rp: bool,

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
  Mobility {
    station: usize,
    crossed: bool,
  },
  Community {
    auto: bool,
    row: usize,
    col: usize,
    gamepiece: GamepieceType
  },
  AutoDocked {
    docked: bool
  },
  ChargeStationLevel {
    auto: bool,
    level: bool
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

// Game Manual 4.4.5
impl LiveScore {
  pub fn new(num_teams: usize) -> Self {
    Self {
      mobility: vec![false; num_teams],
      community: ModeScore {
        auto: vec![vec![GamepieceType::None; 9]; 3],
        teleop: vec![vec![GamepieceType::None; 9]; 3]
      },
      auto_docked: false,
      charge_station_level: ModeScore {
        auto: true, teleop: true
      },
      endgame: vec![EndgameType::None; num_teams],
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
      mobility_points: self.mobility_points(),
      link_points: self.link_points(),
      community_points: self.community_points(),
      auto_docked_points: self.auto_docked_points(),
      endgame_points: self.endgame_points(true),
      meets_coopertition: self.meets_coopertition(),
      sustainability_threshold: self.sustainability_threshold(other_alliance),
      sustainability_rp: self.sustainability_rp(other_alliance),
      activation_rp: self.activation_rp(),
      
      mode_score,
      penalty_score: penalty_points,
      total_score,
      total_bonus_rp: self.total_bonus_rp(other_alliance),
      win_rp,
      total_rp: self.total_bonus_rp(other_alliance) + win_rp,
      win_status,
    }
  }

  // TODO: This needs better error handling - currently all inputs are assumed to be correct
  pub fn update(&mut self, score_update: ScoreUpdate) {
    match score_update {
      ScoreUpdate::Mobility { station, crossed } => {
        self.mobility[station] = crossed;
      },
      ScoreUpdate::Community { auto, row, col, gamepiece } => {
        if auto {
          self.community.auto[row][col] = gamepiece;
        } else {
          self.community.teleop[row][col] = gamepiece;
        }
      },
      ScoreUpdate::AutoDocked { docked } => {
        self.auto_docked = docked;
      },
      ScoreUpdate::ChargeStationLevel { auto, level } => {
        if auto {
          self.charge_station_level.auto = level;
        } else {
          self.charge_station_level.teleop = level;
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

  fn mobility_points(&self) -> isize {
    self.mobility.iter().map(|&x| (x as isize) * 3).sum()
  }

  fn auto_docked_points(&self) -> isize {
    if self.charge_station_level.auto {
      self.auto_docked as isize * 12
    } else {
      self.auto_docked as isize * 8
    }
  }

  fn community_points(&self) -> ModeScore<isize> {
    let mut ms = ModeScore { auto: 0, teleop: 0 };

    for (row, cols) in self.community.auto.iter().enumerate() {
      let point_base = match row {
        0 => 3,
        1 => 4,
        2 => 6,
        _ => panic!("Unknown community row")
      };

      ms.auto = point_base * cols.iter().filter(|&x| *x != GamepieceType::None).count() as isize;
    }

    for (row, cols) in self.community.teleop.iter().enumerate() {
      let point_base = match row {
        0 => 2,
        1 => 3,
        2 => 5,
        _ => panic!("Unknown community row")
      };

      ms.teleop = point_base * cols.iter().filter(|&x| *x != GamepieceType::None).count() as isize;
    }

    ms
  }

  fn link_count(&self) -> isize {
    let mut occupancy_grid = vec![vec![false; 9]; 3];
    for (row, cols) in self.community.auto.iter().enumerate() {
      for (col, gptype) in cols.iter().enumerate() {
        if *gptype != GamepieceType::None {
          occupancy_grid[row][col] = true;
        }
      }
    }

    let mut n_links = 0;

    // Scan each row left to right
    for row in 0..occupancy_grid.len() {
      let mut col = 0;
      let mut count = 0;
      while col < occupancy_grid[row].len() {
        if occupancy_grid[row][col] {
          count += 1;
        }
        if count == 3 {
          count = 0;
          n_links += 1;
        }
        col += 1;
      }
    }

    n_links
  }

  fn link_points(&self) -> isize {
    self.link_count() * 5
  }

  fn meets_coopertition(&self) -> bool {
    let mut total = 0;
    for els in self.community.auto.iter().chain(self.community.teleop.iter()) {
      // At least three in the center grid
      for col in 3..=5 {
        if els[col] != GamepieceType::None {
          total += 1;
        }
      }
    }
    total >= 3
  }

  fn sustainability_threshold(&self, other_alliance: &Self) -> usize {
    if self.meets_coopertition() && other_alliance.meets_coopertition() {
      4
    } else {
      5
    }
  }

  fn sustainability_rp(&self, other_alliance: &Self) -> bool {
    self.link_count() >= self.sustainability_threshold(other_alliance) as isize
  }
  
  fn activation_rp(&self) -> bool {
    (self.auto_docked_points() + self.endgame_points(false)) >= 26
  }

  fn endgame_points(&self, allow_parked: bool) -> isize {
    let level = self.charge_station_level.teleop;

    self.endgame.iter().map(|x| match x {
      EndgameType::None => 0,
      EndgameType::Parked if !allow_parked => 0,
      EndgameType::Parked => 2,
      EndgameType::Docked if level => 10,
      EndgameType::Docked => 6,
    }).sum()
  }

  fn penalty_points_other_alliance(&self) -> usize {
    self.penalties.fouls * 5 + self.penalties.tech_fouls * 12
  }

  fn mode_score(&self) -> ModeScore<isize> {
    ModeScore {
      auto: self.mobility_points() + self.community_points().auto + self.auto_docked_points(),
      teleop: self.community_points().teleop + self.link_points() + self.endgame_points(true),
    }
  }

  fn total_bonus_rp(&self, other_alliance: &Self) -> usize {
    self.sustainability_rp(other_alliance) as usize + self.activation_rp() as usize
  }
}
