use jms_macros::define_websocket_msg;

use crate::{models, db::{self, TableType}};

use super::{ws::{WebsocketContext, WebsocketHandler, Websocket}, WebsocketMessage2JMS};

define_websocket_msg!($TicketMessage {
  recv Insert(models::SupportTicket),
  send All(Vec<models::SupportTicket>),
});

pub struct WSTicketHandler;

#[async_trait::async_trait]
impl WebsocketHandler for WSTicketHandler {
  async fn broadcast(&self, ctx: &WebsocketContext) -> anyhow::Result<()> {
    ctx.broadcast::<TicketMessage2UI>(TicketMessage2UI::All( models::SupportTicket::all(&db::database())? ).into());
    Ok(())
  }

  async fn handle(&self, msg: &WebsocketMessage2JMS, _ws: &mut Websocket) -> anyhow::Result<()> {
    if let WebsocketMessage2JMS::Ticket(msg) = msg {
      match msg.clone() {
        TicketMessage2JMS::Insert(mut ticket) => {
          if ticket.id.is_none() {
            ticket.notes.insert(0, models::TicketComment {
              author: "System".to_owned(),
              time: chrono::Local::now().into(),
              comment: format!("Ticket opened by {}", &ticket.author)
            });
          }
          ticket.insert(&db::database())?;
        }
      }
    }
    Ok(())
  }
}