use std::collections::HashMap;

use super::station::AllianceStationId;

pub mod referee {
  use crate::models::Alliance;

  #[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq)]
  #[serde(rename_all="snake_case")]
  pub enum NearFar { Near, Far }

  #[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq)]
  pub enum RefereeID {
    HeadReferee,
    Alliance(Alliance, NearFar)
  }
}

pub mod scorer {
  use crate::models::Alliance;

  #[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq)]
  pub struct ScorerID {
    alliance: Alliance
  }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq)]
pub enum ResourceRole {
  Unknown,
  Any,
  // Panels
  ScorekeeperPanel,
  MonitorPanel,
  RefereePanel(referee::RefereeID),
  ScorerPanel(scorer::ScorerID),
  TimerPanel,
  TeamEStop(AllianceStationId),
  AudienceDisplay,
  // Field Electronics
  FieldElectronics
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Resource {
  pub role: ResourceRole,
  #[serde(default)]
  pub fta: bool,    // For Panels
  #[serde(default)]
  pub ready: bool,
  #[serde(default)]
  pub ready_requested: bool
}

#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct TaggedResource {
  pub id: String,
  #[serde(flatten)]
  pub r: Resource,
}

impl Resource {
  pub fn default(role: ResourceRole) -> Self {
    Self { role, fta: false, ready: false, ready_requested: false }
  }

  pub fn tag(self, id: String) -> TaggedResource {
    TaggedResource { id, r: self }
  }

  pub fn meets_requirements_of(&self, r: &Resource) -> bool {
    (r.role == ResourceRole::Any || self.role == r.role) && self.fta == r.fta
  }

  pub fn ready_satisfied(&self, ready_required: bool) -> bool {
    self.ready || !ready_required
  }
}

#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct Resources(pub HashMap<String, TaggedResource>);

impl Resources {
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  pub fn remove(&mut self, id: &str) -> Option<TaggedResource> {
    self.0.remove(id)
  }

  pub fn register(&mut self, new_id: &str, role: ResourceRole, old_id: &Option<String>) -> &mut TaggedResource {
    if !self.0.contains_key(new_id) {
      if let Some(mut old) = old_id.as_ref().and_then(|id| self.remove(id)) {
        old.id = new_id.to_owned();
        self.0.insert(new_id.to_owned(), old);
      } else {
        self.0.insert(new_id.to_owned(), Resource::default(role).tag(new_id.to_owned()));
      }
    }
    self.get_mut(Some(new_id)).unwrap()
  }

  pub fn get(&self, id: Option<&str>) -> Option<&TaggedResource> {
    id.as_ref().and_then(|id| self.0.get(id.to_owned()))
  }

  pub fn get_mut(&mut self, id: Option<&str>) -> Option<&mut TaggedResource> {
    id.as_ref().and_then(|id| self.0.get_mut(id.to_owned()))
  }

  pub fn all(&self) -> Vec<&TaggedResource> {
    self.0.values().collect()
  }

  pub fn reset_all(&mut self) {
    for (_, v) in self.0.iter_mut() {
      v.r.ready = false;
      v.r.ready_requested = false;
    }
  }
}

// Resource Requirements

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ResourceQuota {
  pub template: Resource,
  pub min: usize,
  pub max: Option<usize>
}

impl ResourceQuota {
  pub fn mapped(&self, resources: &Resources) -> MappedResourceQuota {
    let res = resources.0.values()
      .filter(|&r| r.r.meets_requirements_of(&self.template) );
    
    let n = res.clone().count();
    let min_ok = n >= self.min;
    let max_ok = self.max.map_or(true, |max| n <= max);

    let all_ready = res.clone().all(|r| r.r.ready_satisfied(self.template.ready));

    return MappedResourceQuota { 
      quota: self.clone(), 
      resource_ids: res.map(|r| r.id.clone()).collect(), 
      satisfied: min_ok && max_ok,
      ready: min_ok && max_ok && all_ready
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct MappedResourceQuota {
  #[serde(flatten)]
  quota: ResourceQuota,
  resource_ids: Vec<String>,
  satisfied: bool,
  ready: bool
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum ResourceRequirements {
  And(Vec<Self>),
  Or(Vec<Self>),
  Quota(ResourceQuota)
}

#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub enum ResourceRequirementStatusElement {
  And(Vec<ResourceRequirementStatus>),
  Or(Vec<ResourceRequirementStatus>),
  Quota(MappedResourceQuota)
}

#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct ResourceRequirementStatus {
  pub element: ResourceRequirementStatusElement,
  pub original: ResourceRequirements,
  pub satisfied: bool,
  pub ready: bool
}

impl ResourceRequirements {
  pub fn request_ready(&self, resources: &mut Resources) {
    match self {
      ResourceRequirements::And(all) => {
        for el in all {
          el.request_ready(resources)
        }
      },
      ResourceRequirements::Or(all) => {
        for el in all {
          el.request_ready(resources)
        }
      },
      ResourceRequirements::Quota(q) => {
        for (_, res) in resources.0.iter_mut() {
          if res.r.meets_requirements_of(&q.template) && !res.r.ready_satisfied(q.template.ready) {
            res.r.ready_requested = true;
          }
        }
      },
    }
  }

  pub fn status(self, resources: &Resources) -> ResourceRequirementStatus {
    match self.clone() {
      ResourceRequirements::And(els) => {
        let mapped = els.into_iter().map(|r| r.status(resources));
        ResourceRequirementStatus {
          ready: mapped.clone().all(|r| r.ready),
          satisfied: mapped.clone().all(|r| r.satisfied),
          element: ResourceRequirementStatusElement::And(mapped.collect()),
          original: self
        }
      },
      ResourceRequirements::Or(els) => {
        let mapped = els.into_iter().map(|r| r.status(resources));
        ResourceRequirementStatus {
          ready: mapped.clone().any(|r| r.ready),
          satisfied: mapped.clone().any(|r| r.satisfied),
          element: ResourceRequirementStatusElement::Or(mapped.collect()),
          original: self
        }
      },
      ResourceRequirements::Quota(q) => {
        let mapped = q.mapped(resources);
        ResourceRequirementStatus { 
          ready: mapped.ready, satisfied: mapped.satisfied, 
          element: ResourceRequirementStatusElement::Quota(mapped),
          original: self
        }
      },
    }
  }
}