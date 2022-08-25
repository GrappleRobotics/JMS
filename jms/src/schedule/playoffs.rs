use log::info;

use crate::{db::{self, TableType}, models::{self, AwardRecipient, MatchGenerationRecord, MatchGenerationRecordData, PlayoffAlliance}, schedule::round_robin::round_robin_update};

use super::{bracket::bracket_update, worker::MatchGenerator, GenerationUpdate};

#[derive(Debug, Clone)]
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

    let mut award = models::Award {
      id: None,
      name: award_name.to_owned(),
      recipients: recipients_list
    };
    award.insert(&db::database())?;

    Ok(())
  }
}

impl MatchGenerator for PlayoffMatchGenerator {
  type ParamType = models::PlayoffMode;

  fn match_type(&self) -> crate::models::MatchType {
    models::MatchType::Playoff
  }

  fn generate(
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
      GenerationUpdate::MatchUpdates(updates) => {
        for pending in updates {
          // let alliance_blue = alliances.iter().find(|a| a.id == pending.blue).unwrap();
          // let alliance_red = alliances.iter().find(|a| a.id == pending.red).unwrap();
          let blue_teams = pending.blue.and_then(|id| alliances.iter().find_map(|a| (a.id == id).then(|| a.teams.clone()))).unwrap_or(vec![]);
          let red_teams = pending.red.and_then(|id| alliances.iter().find_map(|a| (a.id == id).then(|| a.teams.clone()))).unwrap_or(vec![]);

          if let Some(mut existing) = models::Match::by_set_match(models::MatchType::Playoff, Some(pending.playoff_type), pending.set, pending.match_num, &db::database())? {
            if !existing.played {
              existing.blue_teams = blue_teams;
              existing.red_teams = red_teams;
              existing.blue_alliance = pending.blue;
              existing.red_alliance = pending.red;
              existing.ready = pending.blue.is_some() && pending.red.is_some();
              existing.insert(&db::database())?;
            }
          } else {
            let mut m = models::Match {
              start_time: None,
              match_type: models::MatchType::Playoff,
              match_subtype: Some(pending.playoff_type),
              set_number: pending.set,
              match_number: pending.match_num,
              blue_teams: blue_teams,
              blue_alliance: pending.blue,
              red_teams: red_teams,
              red_alliance: pending.red,
              score: None,
              score_time: None,
              winner: None,
              played: false,
              ready: pending.blue.is_some() && pending.red.is_some()
            };
            m.insert(&db::database())?;
          }

        }
      }
      GenerationUpdate::TournamentWon(winner, finalist) => {
        info!("Winner: {:?}", winner);
        info!("Finalist: {:?}", finalist);
        self.generate_award_for("Winner", &winner)?;
        self.generate_award_for("Finalist", &finalist)?;
      }
    }

    let mut mgr = MatchGenerationRecord {
      match_type: models::MatchType::Playoff,
      data: Some(MatchGenerationRecordData::Playoff { mode })
    };
    mgr.insert(&db::database())?;

    Ok(())
  }
}
