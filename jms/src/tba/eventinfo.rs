use crate::{db::{self, TableType, DBSingleton}, models};

use super::TBAClient;

#[derive(serde_repr::Serialize_repr, Debug, Clone)]
#[repr(usize)]
#[allow(dead_code)]
pub enum TBAPlayoffType {
  Bracket8 = 0,
  Bracket16 = 1,
  Bracket4 = 2,
  AvgScore8 = 3,
  RoundRobin6 = 4,
  DoubleElim8 = 5,
  BestOf5FinalOnly = 6,
  BestOf3FinalOnly = 7,
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
  pub async fn issue(db: &db::Store, client: &TBAClient) -> anyhow::Result<()> {
    let ed = models::EventDetails::get(db)?;
    if let Some(code) = ed.code {
      let update = TBAEventInfoUpdate {
        first_code: Some(code[4..].to_owned()),   // Substring away the year
        playoff_type: Self::playoff_type(db)?.unwrap_or(TBAPlayoffType::Bracket8),
        webcasts: ed.webcasts.iter().map(|wc| TBAWebcast { url: wc.clone() }).collect()
      };

      client.post("info", "update", &update).await?;
    }
    Ok(())
  }

  fn playoff_type(db: &db::Store) -> anyhow::Result<Option<TBAPlayoffType>> {
    let mgr = models::MatchGenerationRecord::get(models::MatchType::Playoff, db)?;
    
    Ok(match mgr.data {
      Some(models::MatchGenerationRecordData::Playoff { mode }) => {
        let num_alliances = models::PlayoffAlliance::table(db)?.len();

        Some(match (mode, num_alliances) {
          (models::PlayoffMode::Bracket, 2) => TBAPlayoffType::BestOf3FinalOnly,
          (models::PlayoffMode::Bracket, 3..=4) => TBAPlayoffType::Bracket4,
          (models::PlayoffMode::Bracket, 5..=8) => TBAPlayoffType::Bracket8,
          (models::PlayoffMode::Bracket, 9..=16) => TBAPlayoffType::Bracket16,
          // (models::PlayoffMode::RoundRobin, 6) => TBAPlayoffType::RoundRobin6,
          (models::PlayoffMode::DoubleBracket, _) => TBAPlayoffType::DoubleElim8,
          _ => TBAPlayoffType::Custom
        })
      },
      _ => None,
    })
  }
}