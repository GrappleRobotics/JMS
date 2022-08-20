use std::convert::TryFrom;

use anyhow::bail;

use crate::{config::Interactive, db::{self, TableType}, models};

pub mod eventinfo;
pub mod alliances;
pub mod awards;
pub mod rankings;
pub mod teams;
mod matches;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct TBAClient {
  pub base_url: String,
  pub auth_id: String,
  pub auth_secret: String,
}

impl TBAClient {
  pub async fn post<T>(&self, noun: &str, verb: &str, data: &T) -> anyhow::Result<()>
    where T: serde::Serialize
  {
    // TODO: Need a way to 'debounce' calls (i.e. large team imports)
    let code = models::EventDetails::get(&db::database())?.code;
    if let Some(code) = code {
      let fragment = format!("/api/trusted/v1/event/{}/{}/{}", code, noun, verb);
      let data = serde_json::to_string(data)?;
      let md5_in = format!("{}{}{}", self.auth_secret, fragment, data);
      let md5_str = format!("{:x}", md5::compute(md5_in));

      info!("TBA Update: {} -> data: {}", fragment, data);
      let response = reqwest::Client::new()
        .post(format!("{}{}", self.base_url, fragment))
        .header("X-TBA-Auth-Id", &self.auth_id)
        .header("X-TBA-Auth-Sig", md5_str)
        .body(data)
        .send()
        .await?;
      
      response.error_for_status()?;
      Ok(())
    } else {
      bail!("Can't issue a TBA Update if the Event doesn't have a sync code. Add one in the Event Wizard.");
    }
  }
}

pub struct TBAWorker {
  pub client: TBAClient
}

impl TBAWorker {
  pub fn new(client: TBAClient) -> Self {
    Self { client }
  }

  // TBA Errors shouldn't stop the whole system
  pub async fn begin(&self) -> anyhow::Result<()> {
    if let Err(e) = self.run().await {
      error!("TBA Fatal Error: {}", e);
    }
    Ok(())
  }

  pub async fn run(&self) -> anyhow::Result<()> {
    let db = db::database();
    let mut details = models::EventDetails::table(db)?.watch_all();
    let mut teams = models::Team::table(db)?.watch_all();
    let mut matches = models::Match::table(db)?.watch_all();
    let mut rankings = models::TeamRanking::table(db)?.watch_all();
    let mut alliances = models::PlayoffAlliance::table(db)?.watch_all();
    let mut match_records = models::MatchGenerationRecord::table(db)?.watch_all();
    // Awards aren't included - TBA can't handle non-standard award names

    loop {
      tokio::select! {
        event = details.get() => {
          event?;
          if let Err(e) = eventinfo::TBAEventInfoUpdate::issue(db, &self.client).await {
            error!("TBA Event Info Error: {}", e);
          }
        },
        event = match_records.get() => {
          event?;
          // Match Generation Records describe the playoff schedule, which
          // is a part of the TBA Event Info update
          if let Err(e) = eventinfo::TBAEventInfoUpdate::issue(db, &self.client).await {
            error!("TBA Event Info Error: {}", e);
          }
        },
        event = teams.get() => {
          event?;
          if let Err(e) = teams::TBATeams::issue(db, &self.client).await {
            error!("TBA Team Error: {}", e);
          }
        },
        event = matches.get() => {
          match event? {
            db::WatchEvent::Insert(m) => {
              let mut tba_match = matches::TBAMatch::try_from(m.data.clone())?;
              match tba_match.issue(&self.client).await {
                Ok(_) => (),
                Err(_) => {
                  // Try again without the score breakdown
                  info!("Trying match upload again without score breakdown...");
                  tba_match.score_breakdown = None;
                  if let Err(e) = tba_match.issue(&self.client).await {
                    error!("TBA Match Error: {}", e);
                  }
                },
              }
            },
            db::WatchEvent::Remove(key) => matches::TBAMatch::delete(key, &self.client).await?,
          }
        },
        event = rankings.get() => {
          event?;
          let ranks = models::TeamRanking::all(&db::database())?;
          if let Err(e) = rankings::TBARankings::from(ranks).issue(&self.client).await {
            error!("TBA Rankings Error: {}", e);
          }
        },
        event = alliances.get() => {
          event?;
          let alls = models::PlayoffAlliance::all(&db::database())?;
          if let Err(e) = alliances::TBAAlliances::from(alls).issue(&self.client).await {
            error!("TBA Alliances Error: {}", e);
          }
        }
      }
    }
  }
}

#[async_trait::async_trait]
impl Interactive for TBAClient {
  async fn interactive() -> anyhow::Result<Self> {
    let base_url = inquire::Text::new("Base URL")
      .with_default("https://www.thebluealliance.com")
      .prompt()?;
    
      let auth_id = inquire::Text::new("Trusted API Auth ID").prompt()?;
      let auth_secret = inquire::Text::new("Trusted API Auth Secret").prompt()?;

      Ok(TBAClient {
        base_url,
        auth_id,
        auth_secret
      })
  }
}