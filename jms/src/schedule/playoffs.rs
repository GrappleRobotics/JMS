use log::info;

use crate::{db::{self, TableType}, models::{self, AwardRecipient, MatchGenerationRecord, MatchGenerationRecordData, PlayoffAlliance}, schedule::round_robin::round_robin_update};

use super::{bracket::bracket_update, worker::MatchGenerator, GenerationUpdate};

#[derive(Clone)]
pub struct PlayoffMatchGenerator;

impl PlayoffMatchGenerator {
  pub fn new() -> Self {
    Self {}
  }

  fn generate_award_for(&self, award_name: &str, alliance: &PlayoffAlliance) -> Result<(), Box<dyn std::error::Error>> {
    let recipients_list: Vec<AwardRecipient> = alliance
      .teams
      .iter()
      .filter_map(|t| {
        t.map(|x| AwardRecipient {
          team: Some(x),
          awardee: None,
        })
      })
      .collect();

    let award = models::Award {
      id: None,
      name: award_name.to_owned(),
      recipients: recipients_list
    };
    award.insert(&db::database())?;

    Ok(())
  }
}

#[async_trait::async_trait]
impl MatchGenerator for PlayoffMatchGenerator {
  type ParamType = models::PlayoffMode;

  fn match_type(&self) -> crate::models::MatchType {
    models::MatchType::Playoff
  }

  async fn generate(
    &self,
    mode: Self::ParamType,
    record: Option<MatchGenerationRecord>,
  ) -> Result<(), Box<dyn std::error::Error>> {
    let alliances = models::PlayoffAlliance::all(&db::database())?;

    let existing_matches = match record.as_ref() {
      Some(record) => models::Match::by_type(record.match_type, &db::database()).ok(),
      None => None,
    };

    let update = match mode {
      models::PlayoffMode::Bracket => bracket_update(&alliances, &existing_matches),
      models::PlayoffMode::RoundRobin => round_robin_update(&alliances, &existing_matches),
    };

    match update {
      GenerationUpdate::NoUpdate => (),
      GenerationUpdate::NewMatches(pending_matches) => {
        for pending in pending_matches {
          let alliance_blue = alliances.iter().find(|a| a.id == pending.blue).unwrap();
          let alliance_red = alliances.iter().find(|a| a.id == pending.red).unwrap();

          let m = models::Match {
            start_time: None,
            match_type: models::MatchType::Playoff,
            match_subtype: Some(pending.playoff_type),
            set_number: pending.set,
            match_number: pending.match_num,
            blue_teams: alliance_blue.teams.clone(),
            blue_alliance: Some(alliance_blue.id),
            red_teams: alliance_red.teams.clone(),
            red_alliance: Some(alliance_blue.id),
            score: None,
            score_time: None,
            winner: None,
            played: false,
          };
          m.insert(&db::database())?;
        }
      }
      GenerationUpdate::TournamentWon(winner, finalist) => {
        info!("Winner: {:?}", winner);
        info!("Finalist: {:?}", finalist);
        self.generate_award_for("Winner", &winner)?;
        self.generate_award_for("Finalist", &finalist)?;
      }
    }

    let mgr = MatchGenerationRecord {
      match_type: models::MatchType::Playoff,
      data: Some(MatchGenerationRecordData::Playoff { mode })
    };
    mgr.insert(&db::database())?;

    Ok(())
  }
}
