use std::time::{Duration, Instant};

use diesel::RunQueryDsl;
use log::{info, warn};

use crate::{db, models::{self, Alliance, SQLJson}, scoring::scores::MatchScore};

use super::exceptions::{MatchError, MatchResult};

use serde::Serialize;

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

#[derive(Clone, Debug)]
pub struct MatchConfig {
  warmup_cooldown_time: Duration,
  auto_time: Duration,
  pause_time: Duration,
  teleop_time: Duration,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoadedMatch {
  #[serde(rename = "match")]
  match_meta: models::Match,
  state: MatchPlayState,
  remaining_time: Duration,
  pub score: MatchScore,

  #[serde(skip)]
  state_first: bool,
  #[serde(skip)]
  state_start_time: Instant,
  #[serde(skip)]
  config: MatchConfig
}

// TODO: Update match metadata whenever the Arena team list changes (arena is able to swap teams out)

impl LoadedMatch {
  pub fn new(m: models::Match) -> LoadedMatch {
    LoadedMatch {
      state: MatchPlayState::Waiting,
      score: MatchScore::new( m.red_teams.0.len(), m.blue_teams.0.len() ),
      match_meta: m,
      state_first: true,
      state_start_time: Instant::now(),
      remaining_time: Duration::from_secs(0),
      config: MatchConfig {
        warmup_cooldown_time: Duration::from_secs(3),
        auto_time: Duration::from_secs(4),
        pause_time: Duration::from_secs(1),
        teleop_time: Duration::from_secs(4),
      },
    }
  }

  pub fn current_state(&self) -> MatchPlayState {
    self.state
  }

  pub fn metadata(&self) -> &models::Match {
    &self.match_meta
  }

  pub fn start(&mut self) -> MatchResult<()> {
    if self.state == MatchPlayState::Waiting {
      self.do_change_state(MatchPlayState::Warmup);
      Ok(())
    } else {
      Err(MatchError::IllegalStateChange {
        from: self.state,
        to: MatchPlayState::Waiting,
        why: "Match not ready!".to_owned(),
      })
    }
  }

  pub async fn commit_score(&mut self) -> MatchResult<()> {
    if self.match_meta.match_type != models::MatchType::Test {
      if self.state == MatchPlayState::Complete {
        let red = self.score.red.derive();
        let blue = self.score.blue.derive();

        let mut winner = None;
        if blue.total_score.total() > red.total_score.total() {
          winner = Some(Alliance::Blue);
        } else if red.total_score.total() > blue.total_score.total() {
          winner = Some(Alliance::Red);
        }

        self.match_meta.played = true;
        self.match_meta.winner = winner;
        self.match_meta.score = Some(SQLJson(self.score.clone()));

        {
          use crate::schema::matches::dsl::*;
          diesel::replace_into(matches).values(&self.match_meta).execute(&db::connection()).unwrap();
        }

        if self.match_meta.match_type == models::MatchType::Qualification {
          let conn = db::connection();
          for &team in &self.match_meta.blue_teams.0 {
            if let Some(team) = team {
              models::TeamRanking::get(team, &conn)?.update( &blue, &red, &conn )?;
            }
          }
          for &team in &self.match_meta.red_teams.0 {
            if let Some(team) = team {
              models::TeamRanking::get(team, &conn)?.update( &red, &blue, &conn )?;
            }
          }
        }

        Ok(())
      } else {
        Err(MatchError::WrongState { state: self.state, why: "Can't commit score before Match is complete!".to_owned() })
      }
    } else {
      Ok(())
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
      }
      MatchPlayState::Cooldown => {
        self.remaining_time = self.config.warmup_cooldown_time.saturating_sub(elapsed);
        if self.remaining_time == Duration::ZERO {
          self.do_change_state(MatchPlayState::Complete);
        }
      }
      MatchPlayState::Complete => {}
      MatchPlayState::Fault => {
        if first {
          warn!("Match fault");
        }
      }
    }
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