use serde_json::json;

use crate::{config::Interactive, models, db::{TableType, self}};

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
        crate::db::WatchEvent::Insert(ticket) if ticket.new => {
          let t = ticket.data;

          let msg = json!({
            "content": format!("<@&{}> New Ticket Opened by {}. Team: {}, Type: {}, Match: {}. http://10.0.100.5/csa/{}", self.settings.csa_role, t.author, t.team, t.issue_type, t.match_id.unwrap_or("None".to_owned()), t.id.unwrap())
          });
          let result = self.client.send_message(self.settings.csa_channel, &msg).await;
          match result {
            Ok(_) => (),
            Err(e) => error!("Discord CSA Message Error: {}", e)
          }
        },
        _ => ()
      }
    }
  }
}