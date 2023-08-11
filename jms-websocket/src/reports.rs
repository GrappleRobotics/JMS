use jms_core_lib::{models::{MaybeToken, MatchType, Permission}, reports::{ReportGeneratorRPCClient, ReportData}};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait ReportWebsocket {
  #[endpoint]
  async fn awards(&self, ctx: &WebsocketContext, _token: &MaybeToken) -> anyhow::Result<ReportData> {
    ReportGeneratorRPCClient::awards_report(&ctx.mq).await.map_err(|e| anyhow::anyhow!(e))?.map_err(|e| anyhow::anyhow!(e))
  }

  #[endpoint]
  async fn teams(&self, ctx: &WebsocketContext, _token: &MaybeToken) -> anyhow::Result<ReportData> {
    ReportGeneratorRPCClient::teams_report(&ctx.mq).await.map_err(|e| anyhow::anyhow!(e))?.map_err(|e| anyhow::anyhow!(e))
  }

  #[endpoint]
  async fn rankings(&self, ctx: &WebsocketContext, _token: &MaybeToken) -> anyhow::Result<ReportData> {
    ReportGeneratorRPCClient::rankings_report(&ctx.mq).await.map_err(|e| anyhow::anyhow!(e))?.map_err(|e| anyhow::anyhow!(e))
  }

  #[endpoint]
  async fn matches(&self, ctx: &WebsocketContext, _token: &MaybeToken, individual: bool, match_type: MatchType) -> anyhow::Result<ReportData> {
    ReportGeneratorRPCClient::match_report(&ctx.mq, match_type, individual).await.map_err(|e| anyhow::anyhow!(e))?.map_err(|e| anyhow::anyhow!(e))
  }

  #[endpoint]
  async fn wpa_key(&self, ctx: &WebsocketContext, token: &MaybeToken, csv: bool) -> anyhow::Result<ReportData> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    ReportGeneratorRPCClient::wpa_report(&ctx.mq, csv).await.map_err(|e| anyhow::anyhow!(e))?.map_err(|e| anyhow::anyhow!(e))
  }
}