mod reports;

pub async fn begin() -> anyhow::Result<()> {
  rocket::build()
    .mount("/reports", routes![reports::teams, reports::rankings, reports::matches_per_team, reports::matches, reports::awards, reports::wpa])
    .launch()
    .await?;
  Ok(())
}