mod reports;

pub async fn begin() -> anyhow::Result<()> {
  rocket::build()
    .mount("/reports", routes![reports::teams, reports::rankings, reports::matches, reports::awards])
    .launch()
    .await?;
  Ok(())
}