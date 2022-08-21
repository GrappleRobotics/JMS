use serde_json::json;

use crate::{config::Interactive, models::{self, SupportTicket}, db::{TableType, self}};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DiscordSettings {
  pub app_id: u64,
  pub token: String,
  pub csa_channel: u64,
  pub csa_role: u64
}

#[async_trait::async_trait]
impl Interactive for DiscordSettings {
  async fn interactive() -> anyhow::Result<Self> {
    let app_id = inquire::CustomType::<u64>::new("Discord Application ID")
      .with_formatter(&|i| format!("{}", i))
      .with_error_message("Not a valid number!")
      .prompt()?;

    let token = inquire::Text::new("Discord Bot Token").prompt()?;
    
    let csa_channel = inquire::CustomType::<u64>::new("CSA Channel")
      .with_formatter(&|i| format!("{}", i))
      .with_error_message("Not a valid number!")
      .prompt()?;
    
    let csa_role = inquire::CustomType::<u64>::new("CSA Role ID")
      .with_formatter(&|i| format!("{}", i))
      .with_error_message("Not a valid number!")
      .prompt()?;

    Ok(Self {
      app_id, token, csa_channel, csa_role
    })
  }
}

pub struct DiscordBot {
  settings: DiscordSettings,
  client: serenity::http::client::Http
}

impl DiscordBot {
  pub fn new(settings: DiscordSettings) -> Self {
    let client = serenity::http::client::Http::new_with_application_id(&settings.token, settings.app_id);
    Self { settings, client }
  }

  pub async fn run(self) -> anyhow::Result<()> {
    let mut watch = models::SupportTicket::table(&db::database())?.watch_all();

    loop {
      match watch.get().await? {
        crate::db::WatchEvent::Insert(ticket) => {
          let t = ticket.data;

          let msg_args = json!({
            "content": DiscordTicketMessage::format(self.settings.csa_role, &t)
          });

          let channel = self.settings.csa_channel;

          match DiscordTicketMessage::get(t.id.unwrap(), &db::database())? {
            Some(past_msg) => {
              match self.client.edit_message(channel, past_msg.message_id, &msg_args).await {
                Ok(_) => (),
                Err(e) => {
                  error!("Could not edit message: {}", e);
                  if e.to_string().contains("Unknown Message") {
                    // Remove the message - it's been deleted
                    past_msg.remove(&db::database())?;
                    warn!("Message Record Deleted for Ticket - {}", t.id.unwrap())
                  }
                },
              }
            },
            None => {
              match self.client.send_message(channel, &msg_args).await {
                Ok(reply) => {
                  DiscordTicketMessage {
                    ticket_id: t.id.unwrap(),
                    message_id: reply.id.0
                  }.insert(&db::database())?;
                },
                Err(e) => error!("Could not send message: {}", e)
              } 
            }
          };
        },
        _ => ()
      }
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DiscordTicketMessage {
  pub ticket_id: u64,
  pub message_id: u64
}

impl DiscordTicketMessage {
  fn format(role: u64, t: &SupportTicket) -> String {
    let note = t.notes.last().map(|n| n.comment.clone()).unwrap_or("".to_owned());
    let emoji = match (t.resolved, &t.assigned_to) {
      (true, _) => "✅".to_owned(),
      (false, None) => "[⚠️ NEW]".to_owned(),
      (false, Some(person)) => format!("[👤{}]", person)
    };

    let mut msg = vec![
      format!("<@&{}> {} Team {} - {}", role, emoji, t.team, t.issue_type)
    ];
    if note.trim().len() > 0 {
      msg.push(format!("\"{}\"", note.trim()));
    }
    if let Some(mid) = &t.match_id {
      msg.push(format!("Match {}", mid));
    }
    msg.push(format!("http://10.0.100.5/csa/{}", t.id.unwrap()));

    msg.join(". ")
  }
}

impl TableType for DiscordTicketMessage {
  const TABLE: &'static str = "discord_ticket_msgs";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    Some(self.ticket_id.into())
  }
}