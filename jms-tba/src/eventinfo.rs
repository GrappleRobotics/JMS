use jms_base::kv;
use jms_core_lib::{models::{self, PlayoffModeType}, db::Singleton};

use crate::client::TBAClient;

#[derive(serde_repr::Serialize_repr, Debug, Clone)]
#[repr(usize)]
#[allow(dead_code)]
pub enum TBAPlayoffType {
  Bracket8 = 0,
  Bracket16 = 1,
  Bracket4 = 2,
  AvgScore8 = 3,
  RoundRobin6 = 4,
  LegacyDoubleElim8 = 5,
  BestOf5FinalOnly = 6,
  BestOf3FinalOnly = 7,
  DoubleElim8 = 10,
  Custom = 8
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct TBAWebcast {
  pub url: String
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct TBAEventInfoUpdate {
  pub first_code: Option<String>,
  pub playoff_type: TBAPlayoffType,
  pub webcasts: Vec<TBAWebcast>
}

impl TBAEventInfoUpdate {
  pub async fn issue(kv: &kv::KVConnection) -> anyhow::Result<()> {
    let ed = models::EventDetails::get(kv)?;
    if let Some(code) = ed.code {
      let update = TBAEventInfoUpdate {
        first_code: Some(code[4..].to_owned()),   // Substring away the year
        playoff_type: Self::playoff_type(kv)?,
        webcasts: ed.webcasts.iter().map(|wc| TBAWebcast { url: wc.clone() }).collect()
      };

      TBAClient::post("info", "update", &update, kv).await?;
    }
    Ok(())
  }

  fn playoff_type(kv: &kv::KVConnection) -> anyhow::Result<TBAPlayoffType> {
    let pm = models::PlayoffMode::get(kv)?;
    
    Ok(match (pm.mode, pm.n_alliances) {
      (PlayoffModeType::Bracket, 1..=8) => TBAPlayoffType::Bracket8,
      (PlayoffModeType::DoubleBracket, 1..=8) => TBAPlayoffType::DoubleElim8,
      _ => TBAPlayoffType::Custom
    })
  }
}