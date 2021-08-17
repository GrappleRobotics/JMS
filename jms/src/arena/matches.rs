use std::time::{Duration, Instant};

use anyhow::{bail, Result};

use diesel::RunQueryDsl;
use log::{info, warn};

use crate::{arena::exceptions::MatchWrongState, db, models::{self, Alliance, MatchGenerationRecordData, SQLDatetime, SQLJson}, schedule::{playoffs::PlayoffMatchGenerator, worker::MatchGenerationWorker}, scoring::scores::{MatchScore, WinStatus}};

use serde::Serialize;

use super::exceptions::MatchIllegalStateChange;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Display, Serialize)]
pub enum MatchPlayState {
  Waiting,
  Warmup,
  Auto,
  Pause,
  Teleop,
  Cooldown,
  Complete,
  Fault, // E-stop, cancelled, etc. Fault is unrecoverable without reloading the match.
}

#[derive(Clone, Debug, Serialize)]
pub struct MatchConfig {
  warmup_cooldown_time: Duration,
  auto_time: Duration,
  pause_time: Duration,
  teleop_time: Duration,
  endgame_time: Duration,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoadedMatch {
  #[serde(rename = "match")]
  pub match_meta: models::Match,
  state: MatchPlayState,
  remaining_time: Duration,
  pub score: MatchScore,

  #[serde(skip)]
  state_first: bool,
  #[serde(skip)]
  state_start_time: Instant,

  config: MatchConfig,
  endgame: bool,
}

impl LoadedMatch {
  pub fn new(m: models::Match) -> LoadedMatch {
    LoadedMatch {
      state: MatchPlayState::Waiting,
      score: MatchScore::new(m.red_teams.0.len(), m.blue_teams.0.len()),
      match_meta: m,
      state_first: true,
      state_start_time: Instant::now(),
      remaining_time: Duration::from_secs(0),
      config: MatchConfig {
        warmup_cooldown_time: Duration::from_secs(3),
        auto_time: Duration::from_secs(15),
        pause_time: Duration::from_secs(1),
        teleop_time: Duration::from_secs(2 * 60 + 15),
        endgame_time: Duration::from_secs(30),
      },
      endgame: false,
    }
  }

  pub fn current_state(&self) -> MatchPlayState {
    self.state
  }

  pub fn metadata(&self) -> &models::Match {
    &self.match_meta
  }

  pub fn start(&mut self) -> Result<()> {
    if self.state == MatchPlayState::Waiting {
      self.do_change_state(MatchPlayState::Warmup);
      Ok(())
    } else {
      bail!(MatchIllegalStateChange {
        from: self.state,
        to: MatchPlayState::Waiting,
        why: "Match not ready!".to_owned(),
      })
    }
  }

  pub async fn commit_score(&mut self) -> Result<Option<models::Match>> {

    if self.state == MatchPlayState::Complete {
      let red = self.score.red.derive(&self.score.blue);
      let blue = self.score.blue.derive(&self.score.red);

      let mut winner = None;
      if blue.win_status == WinStatus::WIN {
        winner = Some(Alliance::Blue);
      } else if red.win_status == WinStatus::WIN {
        winner = Some(Alliance::Red);
      }

      self.match_meta.played = true;
      self.match_meta.winner = winner;
      self.match_meta.score = Some(SQLJson(self.score.clone()));
      self.match_meta.score_time = Some(SQLDatetime(chrono::Local::now().naive_utc()));

      if self.match_meta.match_type != models::MatchType::Test {
        {
          use crate::schema::matches::dsl::*;
          diesel::replace_into(matches)
            .values(&self.match_meta)
            .execute(&db::connection())
            .unwrap();
        }

        if self.match_meta.match_type == models::MatchType::Qualification {
          let conn = db::connection();
          for &team in &self.match_meta.blue_teams.0 {
            if let Some(team) = team {
              models::TeamRanking::get(team, &conn)?.update(&blue, &conn)?;
            }
          }
          for &team in &self.match_meta.red_teams.0 {
            if let Some(team) = team {
              models::TeamRanking::get(team, &conn)?.update(&red, &conn)?;
            }
          }
        } else if self.match_meta.match_type == models::MatchType::Playoff {
          // Update playoff generation
          // TODO: We should use a global worker, but this will do for now.
          let worker = MatchGenerationWorker::new(PlayoffMatchGenerator::new());
          let record = worker.record();
          if let Some(record) = record {
            if let Some(MatchGenerationRecordData::Playoff { mode }) = record.data.map(|x| x.0) {
              worker.generate(mode).await;
            }
          }
        }
      }

      Ok(Some(self.match_meta.clone()))
    } else {
      bail!(MatchWrongState {
        state: self.state,
        why: "Can't commit score before Match is complete!".to_owned()
      })
    }
  }

  pub fn fault(&mut self) {
    self.do_change_state(MatchPlayState::Fault);
  }

  // TODO: Implement a self-timing guard for update functions, generating an error if we miss our timing.
  // TODO: Or, async with timeout?
  pub fn update(&mut self) {
    let first = self.state_first;
    self.state_first = false;
    let elapsed = self.elapsed();

    let mut endgame = false;

    match self.state {
      MatchPlayState::Waiting => (),
      MatchPlayState::Warmup => {
        self.remaining_time = self.config.warmup_cooldown_time.saturating_sub(elapsed);
        if self.remaining_time == Duration::ZERO {
          self.do_change_state(MatchPlayState::Auto);
        }
      }
      MatchPlayState::Auto => {
        self.remaining_time = self.config.auto_time.saturating_sub(elapsed);
        if self.remaining_time == Duration::ZERO {
          self.do_change_state(MatchPlayState::Pause);
        }
      }
      MatchPlayState::Pause => {
        self.remaining_time = self.config.pause_time.saturating_sub(elapsed);
        if self.remaining_time == Duration::ZERO {
          self.do_change_state(MatchPlayState::Teleop);
        }
      }
      MatchPlayState::Teleop => {
        self.remaining_time = self.config.teleop_time.saturating_sub(elapsed);
        if self.remaining_time == Duration::ZERO {
          self.do_change_state(MatchPlayState::Cooldown);
        }
        endgame = self.remaining_time <= self.config.endgame_time;
      }
      MatchPlayState::Cooldown => {
        self.remaining_time = self.config.warmup_cooldown_time.saturating_sub(elapsed);
        if self.remaining_time == Duration::ZERO {
          self.do_change_state(MatchPlayState::Complete);
        }
        endgame = true;
      }
      MatchPlayState::Complete => {
        endgame = true;
      }
      MatchPlayState::Fault => {
        if first {
          warn!("Match fault");
        }
      }
    }

    self.endgame = endgame;
  }

  pub fn remaining_time(&self) -> Duration {
    self.remaining_time
  }

  fn do_change_state(&mut self, state: MatchPlayState) {
    if state != self.state {
      info!("Transitioning {:?} -> {:?}...", self.state, state);
      self.state = state;
      self.state_start_time = Instant::now();
      self.state_first = true;
    }
  }

  fn elapsed(&self) -> Duration {
    return Instant::now() - self.state_start_time;
  }
}
