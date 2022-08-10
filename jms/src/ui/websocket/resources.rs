use jms_macros::define_websocket_msg;

use crate::{arena::resource::{TaggedResource, ResourceRole, ResourceRequirementStatus, SharedResources, ResourceRequirements}, models::{FTAKey, DBResourceRequirements}, db::{self, TableType}};

use super::WebsocketMessage2UI;

define_websocket_msg!($ResourceMessage {
  send All(Vec<TaggedResource>),
  send Current(TaggedResource),
  recv SetID(String),
  recv SetRole(ResourceRole),
  recv SetFTA(Option<String>),

  $Requirements {
    send Current(Option<ResourceRequirementStatus>),
    recv SetActive(Option<ResourceRequirements>)
  }
});

pub async fn ws_periodic_resources(s_resources: SharedResources) -> super::Result<Vec<ResourceMessage2UI>> {
  let resources = s_resources.lock().await;
  let mut msgs = vec![];

  msgs.push(ResourceMessage2UI::All(resources.all().into_iter().cloned().collect()));
  {
    let rr = DBResourceRequirements::get(&db::database())?.0;
    msgs.push(ResourceMessageRequirements2UI::Current(rr.map(|r| r.status(&resources))).into());
  }

  Ok(msgs)
}

pub async fn ws_periodic_resources1(s_resources: SharedResources, resource_id: &Option<String>) -> super::Result<Vec<ResourceMessage2UI>> {
  let resources = s_resources.lock().await;

  let mut msgs = vec![];

  if let Some(resource) = resources.get(resource_id.as_deref()) {
    msgs.push(ResourceMessage2UI::Current(resource.clone()))
  }

  Ok(msgs)
}

pub async fn ws_recv_resources(msg: &ResourceMessage2JMS, resources: SharedResources, resource_id: &mut Option<String>) -> super::Result<Vec<WebsocketMessage2UI>> {
  let mut resources = resources.lock().await;
  
  match msg.clone() {
    ResourceMessage2JMS::SetID(id) => {
      resources.register(&id, ResourceRole::Unknown, resource_id);
      *resource_id = Some(id.clone());
    },
    ResourceMessage2JMS::SetRole(role) => {
      if let Some(resource) = resources.get_mut(resource_id.as_deref()) {
        resource.r.role = role;
      }
    },
    ResourceMessage2JMS::SetFTA(key) => {
      if let Some(resource) = resources.get_mut(resource_id.as_deref()) {
        match key {
          Some(key) => {
            if FTAKey::get(&db::database())?.validate(&key) {
              resource.r.fta = true;
            } else {
              resource.r.fta = false;
              anyhow::bail!("Incorrect FTA Key!")
            }
          },
          _ => resource.r.fta = false
        }
      } 
    }
    ResourceMessage2JMS::Requirements(reqmsg) => match reqmsg {
      ResourceMessageRequirements2JMS::SetActive(rr) => {
        DBResourceRequirements(rr).insert(&db::database())?;
      },
    },
  };

  Ok(vec![])
}