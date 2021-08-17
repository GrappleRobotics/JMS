pub mod eventinfo;
pub mod alliances;
pub mod awards;
pub mod rankings;
pub mod teams;
mod matches;

pub struct TBAClient {
  root: String,
  code: String,
  auth_id: String,
  auth_secret: String,
}

impl TBAClient {
  pub fn new(event_code: String, auth_id: String, auth_secret: String) -> Self {
    TBAClient {
      root: "https://www.thebluealliance.com/".to_owned(),
      code: event_code,
      auth_id,
      auth_secret
    }
  }

  pub async fn post<T>(&self, noun: String, verb: String, data: &T) -> anyhow::Result<()>
    where T: serde::Serialize
  {
    let fragment = format!("/api/trusted/v1/event/{}/{}/{}", self.code, noun, verb);
    let data = serde_json::to_string(data)?;
    let md5_in = format!("{}{}{}", self.auth_secret, fragment, data);
    let md5_str = format!("{:x}", md5::compute(md5_in));

    let response = reqwest::Client::new()
      .post(format!("{}{}", self.root, fragment))
      .header("X-TBA-Auth-Id", &self.auth_id)
      .header("X-TBA-Auth-Sig", md5_str)
      .body(data)
      .send()
      .await?;
    
    response.error_for_status()?;
    Ok(())
  }
}
