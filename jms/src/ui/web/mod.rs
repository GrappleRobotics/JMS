mod reports;

pub async fn begin() -> Result<(), Box<dyn std::error::Error>> {
  rocket::build()
    .mount("/reports", routes![reports::teams, reports::rankings, reports::matches, reports::awards])
    .launch()
    .await?;
  Ok(())
}