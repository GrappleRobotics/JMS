use std::convert::Infallible;

use jms_base::kv::KVConnection;
use uuid::Uuid;

use crate::db::{self, Table};

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum Permission {
  Admin,
  /* Roles */
  FTA,
  FTAA,
  Scorekeeper,
  /* Permissions */
  ManageEvent,
  ManageTeams,
  ManageSchedule,
  ManagePlayoffs,
  ManageAwards,
  MatchFlow,
  Estop,
}

impl Permission {
  pub fn has(&self, required: &Permission) -> bool {
    match (self, required) {
      (Permission::Admin, _) => true,
      (a, b) if a == b => true,

      (Permission::FTA, Permission::ManageEvent | Permission::ManageTeams | Permission::ManageSchedule | Permission::ManagePlayoffs |
                        Permission::ManageAwards | Permission::MatchFlow | Permission::Estop) => true,

      (Permission::FTAA, Permission::Estop) => true, 
      (Permission::Scorekeeper, Permission::ManageAwards | Permission::MatchFlow | Permission::Estop) => true,

      _ => false
    }
  }
}

#[derive(jms_macros::Updateable)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct User {
  pub username: String,
  pub realname: String,
  pub pin_hash: Option<String>,
  pub pin_is_numeric: bool,
  pub permissions: Vec<Permission>,
  pub tokens: Vec<String>
}

#[async_trait::async_trait]
impl db::Table for User {
  const PREFIX: &'static str = "db:user";
  type Id = String;
  type Err = Infallible;

  fn id(&self) -> String {
    self.username.clone()
  }
}

impl User {
  pub fn new(username: &str, realname: &str, admin: bool) -> Self {
    Self {
      username: username.to_owned(),
      realname: realname.to_owned(),
      pin_hash: None,
      pin_is_numeric: false,
      permissions: if admin { vec![Permission::Admin] } else { vec![] },
      tokens: vec![]
    }
  }

  // Get case-insensitive
  pub fn get(username: &str, kv: &KVConnection) -> anyhow::Result<Self> {
    let keys = Self::ids(kv)?;
    let key = keys.iter().find(|x| x.to_lowercase() == username.to_lowercase()).ok_or(anyhow::anyhow!("No User Found!"))?;
    kv.json_get(&format!("{}:{}", Self::PREFIX, key), "$")
  }

  pub fn set_pin(&mut self, pin: &str) {
    self.pin_hash = Some(bcrypt::hash(pin.clone(), 10).unwrap());
    self.pin_is_numeric = pin.chars().all(char::is_numeric);
  }

  pub fn pin_auth(&mut self, pin: &str) -> anyhow::Result<UserToken> {
    match &self.pin_hash {
      None => Ok(self.new_token()),
      Some(hash) if bcrypt::verify(pin, hash).unwrap() => Ok(self.new_token()),
      _ => anyhow::bail!("Incorrect PIN")
    }
  }

  pub fn require_permission(&self, permission: &[Permission]) -> anyhow::Result<()> {
    for perm in &self.permissions {
      for required in permission {
        if perm.has(required) {
          return Ok(())
        }
      }
    }
    Err(anyhow::anyhow!("User does not have required permissions!"))
  }

  pub fn has_token(&self, token: &str) -> bool {
    self.tokens.contains(&token.to_owned())
  }

  pub fn new_token(&mut self) -> UserToken {
    let tok = Uuid::new_v4().to_string();

    self.tokens.push(tok.clone());

    UserToken {
      user: self.username.clone(),
      token: tok
    }
  }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct UserToken {
  pub user: String,
  pub token: String
}

pub struct MaybeToken(pub Option<UserToken>);

impl MaybeToken {
  pub fn auth(&self, kv: &KVConnection) -> anyhow::Result<User> {
    match &self.0 {
      Some(token) => {
        match User::get(&token.user, kv) {
          Ok(user) => {
            if user.has_token(&token.token) {
              return Ok(user)
            } else {
              anyhow::bail!("Token is outdated. Please refresh the page.")
            }
          }, 
          Err(_) => anyhow::bail!("Token is for a user who no longer exists. Please refresh the page.")
        }
      },
      None => anyhow::bail!("No token presented! Please refresh the page.")
    }
  }
}