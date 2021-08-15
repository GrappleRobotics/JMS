use std::net::IpAddr;
use rust_embed::RustEmbed;

mod reports;
mod embed;

#[derive(RustEmbed)]
#[folder = "../jms-frontend/build"]
struct WebRoot;

pub async fn begin() -> anyhow::Result<()> {
  let mut default = rocket::config::Config::default();
  default.port = 80;
  default.address = IpAddr::V4("0.0.0.0".parse()?);
  default.shutdown.ctrlc = false;

  rocket::custom(&default)
    .mount("/", embed::EmbedServer::<WebRoot>::new())
    .mount("/reports", routes![reports::teams, reports::rankings, reports::matches_per_team, reports::matches, reports::awards, reports::wpa])
    .launch()
    .await?;
  Ok(())
}