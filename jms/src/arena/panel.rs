use std::collections::HashMap;

use super::station::AllianceStationId;

pub mod referee {
  use crate::models::Alliance;

  #[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
  #[serde(rename_all="snake_case")]
  pub enum NearFar { Near, Far }

  #[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
  pub enum RefereeID {
    HeadReferee,
    Alliance(Alliance, NearFar)
  }
}

pub mod scorer {
  #[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
  #[serde(rename_all="snake_case")]
  pub enum GoalHeight {
    Low, High
  }

  #[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
  pub enum ScorerPair {
    AB, CD
  }

  #[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
  pub struct ScorerID {
    goals: ScorerPair,
    height: GoalHeight
  }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum PanelRole {
  Unknown,
  Scorekeeper,
  Monitor,
  Referee(referee::RefereeID),
  Scorer(scorer::ScorerID),
  Timer,
  EStop(AllianceStationId),
  AudienceDisplay
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Panel {
  pub id: String,
  pub role: PanelRole,
  pub fta: bool
}

impl Panel {
  pub fn default(id: String) -> Self {
    Self { id, role: PanelRole::Unknown, fta: false }
  }
}

#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct Panels(pub HashMap<String, Panel>);

impl Panels {
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  pub fn remove(&mut self, id: &str) -> Option<Panel> {
    let ret = self.0.remove(id);
    self._debug_list_panels();
    ret
  }

  pub fn register(&mut self, new_id: String, old_id: &Option<String>) {
    if !self.0.contains_key(&new_id) {
      if let Some(mut old) = old_id.as_ref().and_then(|id| self.remove(id)) {
        old.id = new_id.clone();
        self.0.insert(new_id, old);
      } else {
        self.0.insert(new_id.clone(), Panel::default(new_id));
      }
      self._debug_list_panels();
    }
  }

  pub fn get(&self, id: &Option<String>) -> Option<&Panel> {
    id.as_ref().and_then(|id| self.0.get(id))
  }

  pub fn get_mut(&mut self, id: &Option<String>) -> Option<&mut Panel> {
    id.as_ref().and_then(|id| self.0.get_mut(id))
  }

  fn _debug_list_panels(&self) {
    debug!("All Panels: {:?}", self.0.iter().map(|(k, v)| (k.as_str(), &v.role) ).collect::<Vec<(&str, &PanelRole)>>())
  }
}

pub type SharedPanels = std::sync::Arc<tokio::sync::Mutex<Panels>>;