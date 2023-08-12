use std::net::{SocketAddr, Ipv4Addr};

use handlebars::Handlebars;
use jms_networking_lib::NetworkingSettings;

use crate::{Resources, ssh::{SSHSession, CommandResult}, NetworkConfig};

pub async fn configure_firewall(config: &NetworkConfig, settings: &NetworkingSettings) -> anyhow::Result<()> {
  let script = generate_script(config).await?;

  let addr = SocketAddr::new(Ipv4Addr::new(10, 0, 100, 1).into(), 22);
  let session = SSHSession::connect(addr, &settings.router_username, &settings.router_password).await?;

  let reply: CommandResult = session.run_with_stdin("pfSsh.php\n", &(script + "\nexit\n")).await?;
  if !reply.success() {
    anyhow::bail!("Failed to set PfSense configuration: {}", reply.output());
  }

  Ok(())
}

async fn generate_script(config: &NetworkConfig) -> anyhow::Result<String> {
  match Resources::get("pfsense_config.php") {
    Some(config_template) => {
      let template_str = std::str::from_utf8(&config_template.data.as_ref())?;
      let hbars = Handlebars::new();

      let result = hbars.render_template(template_str, config)?;
      Ok(result)
    },
    None => anyhow::bail!("No Resource Exists: pfsense_config.php")
  }
}