use std::sync::atomic::AtomicBool;

use jms_base::logging::{LogRecord, meilisearch_uri};
use jms_core_lib::models::{MaybeToken, Permission};
use meilisearch_sdk::settings::Settings;

use crate::ws::WebsocketContext;

static HAS_SET_ATTRS: AtomicBool = AtomicBool::new(false);

pub async fn setup_attrs(client: &meilisearch_sdk::Client) -> anyhow::Result<()> {
  if !HAS_SET_ATTRS.load(std::sync::atomic::Ordering::Relaxed) {
    client.index("logs").set_settings(
      &Settings::new()
        .with_filterable_attributes(["timestamp_utc", "level", "target", "module"])
        .with_sortable_attributes(["timestamp_utc"])
    ).await?;
    HAS_SET_ATTRS.store(true, std::sync::atomic::Ordering::Relaxed);
  }
  Ok(())
}

#[jms_websocket_macros::websocket_handler]
pub trait LogWebsocket {
  #[endpoint]
  async fn get(&self, ctx: &WebsocketContext, token: &MaybeToken, since: Option<f64>) -> anyhow::Result<Vec<LogRecord>> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    let client = meilisearch_sdk::Client::new(meilisearch_uri(), None::<String>);
    setup_attrs(&client).await?;

    let index = client.index("logs");
    let mut search = index.search();
    search.with_sort(&["timestamp_utc:asc"]);
    let results = if let Some(since) = since {
      search.with_filter(&format!("timestamp_utc > {}", since)).execute::<LogRecord>().await?
    } else {
      search.execute::<LogRecord>().await?
    };

    Ok(results.hits.into_iter().map(|x| x.result).collect())
  }

  #[endpoint]
  async fn errors(&self, ctx: &WebsocketContext, token: &MaybeToken, since: f64) -> anyhow::Result<Vec<LogRecord>> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    let client = meilisearch_sdk::Client::new(meilisearch_uri(), None::<String>);
    setup_attrs(&client).await?;

    let results = client.index("logs")
      .search()
      .with_sort(&["timestamp_utc:asc"])
      .with_filter(&format!("level in [ERROR, WARN] AND timestamp_utc > {}", since))
      .execute::<LogRecord>().await?;

    Ok(results.hits.into_iter().map(|x| x.result).collect())
  }
}