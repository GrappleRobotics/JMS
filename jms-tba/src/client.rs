use jms_base::kv;
use jms_core_lib::{models, db::Singleton};
use jms_tba_lib::TBASettings;

use log::{info, debug};

pub struct TBAClient { }

impl TBAClient {
  pub async fn post<T>(noun: &str, verb: &str, data: &T, kv: &kv::KVConnection) -> anyhow::Result<()>
    where T: serde::Serialize
  {
    let code = models::EventDetails::get(kv)?.code;
    let tba_settings = TBASettings::get(kv)?;

    match (code, tba_settings.auth_id, tba_settings.auth_key) {
      (Some(code), Some(auth_id), Some(auth_key)) if code.trim() != "" => {
        let fragment = format!("/api/trusted/v1/event/{}/{}/{}", code, noun, verb);
        let data = serde_json::to_string(data)?;
        let md5_in = format!("{}{}{}", auth_key, fragment, data);
        let md5_str = format!("{:x}", md5::compute(md5_in));

        info!("TBA Update: {} -> data: {}", fragment, data);
        let response = reqwest::Client::new()
          .post(format!("{}{}", "https://www.thebluealliance.com", fragment))
          .header("X-TBA-Auth-Id", auth_id)
          .header("X-TBA-Auth-Sig", md5_str)
          .body(data)
          .send()
          .await?;
        
        response.error_for_status()?;
        Ok(())
      },
      _ => {
        debug!("Can't do TBA Update: missing code or auth");
        Ok(())
      }
    }
  }
}