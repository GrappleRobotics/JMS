use jms_macros::define_websocket_msg;

use crate::{models::{self, MatchStationStatusRecordKey, MatchStationStatusRecord}, db::{self, TableType}};

use super::{ws::{WebsocketContext, WebsocketHandler, Websocket}, WebsocketMessage2JMS};

define_websocket_msg!($TicketMessage {
  recv Insert(models::SupportTicket),
  send All(Vec<models::SupportTicket>),
  
  $Logs {
    recv Keys,
    send Keys(Vec<MatchStationStatusRecordKey>),
    recv Load(MatchStationStatusRecordKey),
    send Load(Option<MatchStationStatusRecord>),
  },
});

pub struct WSTicketHandler;

#[async_trait::async_trait]
impl WebsocketHandler for WSTicketHandler {
  async fn broadcast(&self, ctx: &WebsocketContext) -> anyhow::Result<()> {
    ctx.broadcast::<TicketMessage2UI>(TicketMessage2UI::All( models::SupportTicket::all(&db::database())? ).into());
    Ok(())
  }

  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
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
        },
        TicketMessage2JMS::Logs(msg) => match msg {
          TicketMessageLogs2JMS::Load(key) => {
            let record = MatchStationStatusRecord::get(key, &db::database())?;
            ws.reply(TicketMessage2UI::Logs(TicketMessageLogs2UI::Load(record))).await;
          },
          TicketMessageLogs2JMS::Keys => {
            let keys = MatchStationStatusRecord::keys(&db::database())?;
            ws.reply(TicketMessage2UI::Logs(TicketMessageLogs2UI::Keys(keys.into_iter().map(|k| k.0).collect()))).await;
          }
        }
      }
      self.broadcast(&ws.context).await?;
    }
    Ok(())
  }
}