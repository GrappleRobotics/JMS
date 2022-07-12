use rust_embed::RustEmbed;
use std::net::IpAddr;

mod embed;
mod reports;

#[derive(RustEmbed)]
#[folder = "../jms-frontend/build"]
struct WebRoot;

pub async fn begin(port: u16) -> anyhow::Result<()> {
  let mut default = rocket::config::Config::default();
  default.port = port;
  default.address = IpAddr::V4("0.0.0.0".parse()?);
  default.shutdown.ctrlc = false;

  #[allow(unused_must_use)] {
  rocket::custom(&default)
    .mount("/", embed::EmbedServer::<WebRoot>::new())
    .mount(
      "/reports",
      routes![
        reports::teams,
        reports::rankings,
        reports::matches_per_team,
        reports::matches,
        reports::awards,
        reports::wpa
      ],
    )
    .launch()
    .await?;
  }
  Ok(())
}
