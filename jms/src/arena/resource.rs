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
pub enum ResourceRole {
  Unknown,
  ScorekeeperPanel,
  MonitorPanel,
  RefereePanel(referee::RefereeID),
  ScorerPanel(scorer::ScorerID),
  TimerPanel,
  TeamEStop(AllianceStationId),
  AudienceDisplay
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Resource {
  pub id: String,
  pub role: ResourceRole,
  pub fta: bool,    // For Panels
  pub ready: bool
}

impl Resource {
  pub fn default(id: String) -> Self {
    Self { id, role: ResourceRole::Unknown, fta: false, ready: false }
  }
}

// pub struct PanelRequirements {
//   pub required: 
// }

#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct Resources(pub HashMap<String, Resource>);

impl Resources {
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  pub fn remove(&mut self, id: &str) -> Option<Resource> {
    self.0.remove(id)
  }

  pub fn register(&mut self, new_id: String, old_id: &Option<String>) {
    if !self.0.contains_key(&new_id) {
      if let Some(mut old) = old_id.as_ref().and_then(|id| self.remove(id)) {
        old.id = new_id.clone();
        self.0.insert(new_id, old);
      } else {
        self.0.insert(new_id.clone(), Resource::default(new_id));
      }
    }
  }

  pub fn get(&self, id: &Option<String>) -> Option<&Resource> {
    id.as_ref().and_then(|id| self.0.get(id))
  }

  pub fn get_mut(&mut self, id: &Option<String>) -> Option<&mut Resource> {
    id.as_ref().and_then(|id| self.0.get_mut(id))
  }
}

pub type SharedResources = std::sync::Arc<tokio::sync::Mutex<Resources>>;