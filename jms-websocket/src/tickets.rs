use chrono::Local;
use jms_core_lib::{models::{MaybeToken, SupportTicket, TicketComment, Permission}, db::Table};
use jms_match_logs_lib::MatchLog;
use uuid::Uuid;

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait TicketWebsocket {
  #[endpoint]
  async fn all(&self, ctx: &WebsocketContext, _token: &MaybeToken) -> anyhow::Result<Vec<SupportTicket>> {
    SupportTicket::all(&ctx.kv)
  }

  #[endpoint]
  async fn get(&self, ctx: &WebsocketContext, _token: &MaybeToken, id: String) -> anyhow::Result<SupportTicket> {
    SupportTicket::get(&id, &ctx.kv)
  }

  #[endpoint]
  async fn new(&self, ctx: &WebsocketContext, token: &MaybeToken, team: usize, match_id: Option<String>, issue_type: String) -> anyhow::Result<SupportTicket> {
    let author = token.auth(&ctx.kv)?;
    author.require_permission(&[Permission::Ticketing])?;

    let ticket = SupportTicket {
      id: Uuid::new_v4().to_string(),
      team,
      match_id,
      issue_type,
      author: author.username.clone(),
      notes: vec![TicketComment {
        author: author.username.clone(),
        time: Local::now(),
        comment: format!("Ticket Opened by {}", author.username)
      }],
      assigned_to: None,
      resolved: false
    };

    ticket.insert(&ctx.kv)?;

    Ok(ticket)
  }

  #[endpoint]
  async fn push_comment(&self, ctx: &WebsocketContext, token: &MaybeToken, id: String, comment: String) -> anyhow::Result<SupportTicket> {
    let author = token.auth(&ctx.kv)?;
    author.require_permission(&[Permission::Ticketing])?;
    
    let mut ticket = SupportTicket::get(&id, &ctx.kv)?;
    ticket.notes.push(TicketComment { author: author.username, time: Local::now(), comment });

    ticket.insert(&ctx.kv)?;

    Ok(ticket)
  }

  #[endpoint]
  async fn assign(&self, ctx: &WebsocketContext, token: &MaybeToken, id: String, assign: bool) -> anyhow::Result<SupportTicket> {
    let author = token.auth(&ctx.kv)?;
    author.require_permission(&[Permission::Ticketing])?;

    let mut ticket = SupportTicket::get(&id, &ctx.kv)?;
    ticket.assigned_to = if assign { Some(author.username) } else { None };
    
    ticket.insert(&ctx.kv)?;
    Ok(ticket)
  }

  #[endpoint]
  async fn resolve(&self, ctx: &WebsocketContext, token:  &MaybeToken, id: String, resolve: bool) -> anyhow::Result<SupportTicket> {
    let author = token.auth(&ctx.kv)?;
    author.require_permission(&[Permission::Ticketing])?;

    let mut ticket = SupportTicket::get(&id, &ctx.kv)?;
    ticket.resolved = resolve;
    
    ticket.insert(&ctx.kv)?;
    Ok(ticket)
  }

  #[endpoint]
  async fn get_match_log(&self, ctx: &WebsocketContext, _token: &MaybeToken, match_id: String, team: usize) -> anyhow::Result<MatchLog> {
    MatchLog::get(&format!("{}:{}", match_id, team), &ctx.kv)
  }
}