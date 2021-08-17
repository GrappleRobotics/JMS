#[derive(serde_repr::Serialize_repr, Debug, Clone)]
#[repr(usize)]
pub enum TBAPlayoffType {
  Bracket8 = 0,
  Bracket16 = 1,
  Bracket4 = 2,
  AvgScore8 = 3,
  RoundRobin6 = 4,
  DoubleElim8 = 5,
  BestOf3FinalOnly = 6,
  BestOf5FinalOnly = 7,
  Custom = 8
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct TBAWebcast {
  pub url: String
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct TBAEventInfoUpdate {
  pub first_code: String,
  pub playoff_type: Option<TBAPlayoffType>,
  pub webcasts: Vec<TBAWebcast>
}