use jms_core_lib::{models::{MaybeToken, User, UserToken, Permission}, db::Table};

use crate::ws::WebsocketContext;

#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "type")]
pub enum AuthResult {
  AuthSuccess { user: User, token: UserToken },
  AuthSuccessNewPin { user: User, token: UserToken },
  NoToken,
}

#[jms_websocket_macros::websocket_handler]
pub trait UserWebsocket {
  #[endpoint]
  async fn auth_with_token(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<AuthResult> {
    if User::ids(&ctx.kv)?.is_empty() {
      // Create the default FTA User since there are no current users
      let mut user = User::new("FTA", "FTA", true);
      let utoken = user.new_token();
      user.insert(&ctx.kv)?;  // Make sure the user gets the new token

      Ok(AuthResult::AuthSuccessNewPin { user, token: utoken })
    } else if let Some(utoken) = &token.0 {
      // User has a token - log them in
      let user = token.auth(&ctx.kv)?;
      if user.pin_hash.is_none() {
        Ok(AuthResult::AuthSuccessNewPin { user, token: utoken.clone() })
      } else {
        Ok(AuthResult::AuthSuccess { user, token: utoken.clone() })
      }
    } else {
      // User didn't present a token - they're a guest
      Ok(AuthResult::NoToken)
    }
  }

  #[endpoint]
  async fn auth_with_pin(&self, ctx: &WebsocketContext, _tok: &MaybeToken, username: String, pin: String) -> anyhow::Result<AuthResult> {
    let mut user = User::get(&username, &ctx.kv).map_err(|_e| anyhow::anyhow!("No User with that username"))?;
    let token = user.pin_auth(&pin)?;
    user.insert(&ctx.kv)?;  // Make sure the user gets the new token

    if user.pin_hash.is_none() {
      Ok(AuthResult::AuthSuccessNewPin { user, token })
    } else {
      Ok(AuthResult::AuthSuccess { user, token })
    }
  }

  #[endpoint]
  async fn update_pin(&self, ctx: &WebsocketContext, token: &MaybeToken, pin: String) -> anyhow::Result<User> {
    let mut user = token.auth(&ctx.kv)?;
    user.set_pin(&pin);
    user.insert(&ctx.kv)?;
    return Ok(user)
  }
  
  #[endpoint]
  async fn logout(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<()> {
    let mut user = token.auth(&ctx.kv)?;
    let index = user.tokens.iter().position(|x| x == &token.0.as_ref().unwrap().token).unwrap();
    user.tokens.remove(index);
    user.insert(&ctx.kv)?;
    Ok(())
  }

  /* USER MANAGEMENT */

  #[endpoint]
  async fn users(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<Vec<User>> {
    token.auth(&ctx.kv)?.require_permission(&Permission::Admin)?;
    Ok(User::all(&ctx.kv)?)
  }

  #[endpoint]
  async fn modify_user(&self, ctx: &WebsocketContext, token: &MaybeToken, user: User) -> anyhow::Result<()> {
    let tok_user = token.auth(&ctx.kv)?;
    tok_user.require_permission(&Permission::Admin)?;

    if tok_user.id() == user.id() {
      if tok_user.permissions.contains(&Permission::Admin) && !user.permissions.contains(&Permission::Admin) {
        anyhow::bail!("Can't remove admin from yourself!");
      }
    }

    user.insert(&ctx.kv)?;
    Ok(())
  }

  #[endpoint]
  async fn delete_user(&self, ctx: &WebsocketContext, token: &MaybeToken, user_id: String) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&Permission::Admin)?;
    User::delete_by(&user_id, &ctx.kv)?;
    Ok(())
  }
}