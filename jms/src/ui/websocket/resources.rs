use jms_macros::define_websocket_msg;

use crate::{arena::resource::{TaggedResource, ResourceRole, ResourceRequirementStatus, SharedResources, ResourceRequirements}, models::{FTAKey, DBResourceRequirements}, db::{self, DBSingleton}};

use super::{ws::{WebsocketHandler, WebsocketContext, Websocket}, WebsocketMessage2JMS};

define_websocket_msg!($ResourceMessage {
  send All(Vec<TaggedResource>),
  send Current(TaggedResource),
  recv SetID(String),
  send SetIDACK(String),
  recv SetRole(ResourceRole),
  recv SetFTA(Option<String>),
  recv SetReady(bool),

  $Requirements {
    send Current(Option<ResourceRequirementStatus>),
    recv SetActive(Option<ResourceRequirements>)
  }
});

pub struct WSResourceHandler(pub SharedResources);

#[async_trait::async_trait]
impl WebsocketHandler for WSResourceHandler {
  async fn broadcast(&self, ctx: &WebsocketContext) -> anyhow::Result<()> {
    let resources = self.0.lock().await;

    ctx.broadcast(ResourceMessage2UI::All(resources.all().into_iter().cloned().collect())).await;
    {
      let rr = DBResourceRequirements::get(&db::database())?.0;
      ctx.broadcast(ResourceMessage2UI::from(ResourceMessageRequirements2UI::Current(rr.map(|r| r.status(&resources))))).await;
    }
    Ok(())
  }

  async fn unicast(&self, ws: &Websocket) -> anyhow::Result<()> {
    if let Some(resource) = ws.resource().await {
      ws.send(ResourceMessage2UI::Current(resource.clone())).await;
    }

    Ok(())
  }

  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
    if let WebsocketMessage2JMS::Resource(msg) = msg {
      let mut resources = self.0.lock().await;

      match msg.clone() {
        ResourceMessage2JMS::SetID(id) => {
          resources.register(&id, ResourceRole::Unknown, &ws.resource_id);
          ws.resource_id = Some(id.clone());
          info!("SetID {}", &id); 
          ws.reply(ResourceMessage2UI::SetIDACK(id.clone())).await;
        },
        ResourceMessage2JMS::SetRole(role) => {
          if let Some(resource) = ws.resource_mut(&mut resources) {
            resource.r.role = role;
          }
        },
        ResourceMessage2JMS::SetFTA(key) => {
          if let Some(resource) = ws.resource_mut(&mut resources) {
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
        ResourceMessage2JMS::SetReady(ready) => {
          if let Some(resource) = ws.resource_mut(&mut resources) {
            resource.r.ready = ready;
          }
        },
        ResourceMessage2JMS::Requirements(reqmsg) => {
          match reqmsg {
            ResourceMessageRequirements2JMS::SetActive(rr) => {
              DBResourceRequirements(rr).insert(&db::database())?;
            },
          };
        }
      };

      // Update the client on any changes
      drop(resources);
      self.unicast(&ws).await?;
      self.broadcast(&ws.context).await?;
    }
    Ok(())
  }
}
