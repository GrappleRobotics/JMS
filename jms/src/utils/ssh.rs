use std::{io::Read, net::ToSocketAddrs};

use anyhow::{Result, anyhow};

pub struct SSHSession {
  session: ssh2::Session
}

impl SSHSession {
  pub async fn connect(addr: impl ToSocketAddrs, user: &str, password: &str) -> Result<SSHSession> {
    let addr = addr.to_socket_addrs()?.next().ok_or(anyhow!("Invalid Address"))?;
    let user = user.to_owned();
    let password = password.to_owned();

    tokio::spawn(async move {
      let tcp = std::net::TcpStream::connect(addr)?;
      let mut session = ssh2::Session::new()?;
      session.set_tcp_stream(tcp);
      session.handshake()?;
      session.userauth_password(&user, &password)?;

      Ok::<SSHSession, anyhow::Error>(SSHSession { session })
    }).await?
  }

  pub async fn run(&self, command: &str) -> Result<CommandResult> {
    let command = command.to_owned();
    let session = self.session.clone();

    tokio::spawn(async move {
      let mut channel = session.channel_session()?;
      channel.exec(&command)?;

      let mut s = String::new();
      channel.read_to_string(&mut s)?;
      channel.wait_close()?;

      Ok(CommandResult { output: s, code: Some(channel.exit_status()?) })
    }).await?
  }
}

#[derive(Clone, Debug)]
pub struct CommandResult {
  output: String,
  code: Option<i32>,
}

impl CommandResult {
  pub fn output(&self) -> String {
    self.output.clone()
  }

  pub fn success(&self) -> bool {
    self.code() == Some(0)
  }

  pub fn code(&self) -> Option<i32> {
    self.code
  }
}
