use diesel::{BelongingToDsl, ExpressionMethods, RunQueryDsl};
use log::info;

use crate::{
  db,
  models::{self, AwardRecipient, Match, MatchGenerationRecord, MatchGenerationRecordData, PlayoffAlliance, SQLJson},
  schedule::round_robin::round_robin_update,
};

use super::{bracket::bracket_update, worker::MatchGenerator, GenerationUpdate};

#[derive(Clone)]
pub struct PlayoffMatchGenerator;

impl PlayoffMatchGenerator {
  pub fn new() -> Self {
    Self {}
  }

  fn generate_award_for(&self, award_name: &str, alliance: &PlayoffAlliance) -> Result<(), Box<dyn std::error::Error>> {
    use crate::schema::awards::dsl::*;

    let recipients_list: Vec<AwardRecipient> = alliance
      .teams
      .0
      .iter()
      .filter_map(|t| {
        t.map(|x| AwardRecipient {
          team: Some(x),
          awardee: None,
        })
      })
      .collect();

    diesel::insert_into(awards)
      .values((name.eq(award_name), recipients.eq(SQLJson(recipients_list))))
      .execute(&db::connection())?;

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
    let alliances = {
      use crate::schema::playoff_alliances::dsl::*;
      playoff_alliances.load::<PlayoffAlliance>(&db::connection())?
    };

    let existing_matches = record.and_then(|r| models::Match::belonging_to(&r).load::<Match>(&db::connection()).ok());

    let update = match mode {
      models::PlayoffMode::Bracket => bracket_update(&alliances, &existing_matches),
      models::PlayoffMode::RoundRobin => round_robin_update(&alliances, &existing_matches),
    };

    match update {
      GenerationUpdate::NoUpdate => (),
      GenerationUpdate::NewMatches(pending_matches) => {
        use crate::schema::matches::dsl::*;

        let mut match_vec = vec![];
        for pending in pending_matches {
          let alliance_blue = &alliances.iter().find(|a| a.id == pending.blue).unwrap();
          let alliance_red = &alliances.iter().find(|a| a.id == pending.red).unwrap();

          match_vec.push((
            match_type.eq(models::MatchType::Playoff),
            match_subtype.eq(pending.playoff_type),
            set_number.eq(pending.set),
            match_number.eq(pending.match_num),
            red_alliance.eq(pending.red),
            blue_alliance.eq(pending.blue),
            red_teams.eq(alliance_red.teams.clone()),
            blue_teams.eq(alliance_blue.teams.clone()),
            played.eq(false),
          ));
        }

        diesel::insert_into(matches)
          .values(&match_vec)
          .execute(&*db::connection())?;
      }
      GenerationUpdate::TournamentWon(winner, finalist) => {
        info!("Winner: {:?}", winner);
        info!("Finalist: {:?}", finalist);
        self.generate_award_for("Winner", &winner)?;
        self.generate_award_for("Finalist", &finalist)?;
      }
    }

    {
      use crate::schema::match_generation_records::dsl::*;

      diesel::replace_into(match_generation_records)
        .values(MatchGenerationRecord {
          match_type: models::MatchType::Playoff,
          data: Some(SQLJson(MatchGenerationRecordData::Playoff { mode })),
        })
        .execute(&db::connection())?;
    }

    Ok(())
  }
}
