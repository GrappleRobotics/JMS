use crate::models::MatchType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ReportData {
  data: Vec<u8>,
  mime: String
}

impl ReportData {
  pub fn pdf(data: Vec<u8>) -> Self { Self { data, mime: "application/pdf".to_owned() } }
  pub fn csv(data: Vec<u8>) -> Self { Self { data, mime: "text/csv".to_owned() } }
}

#[jms_macros::service]
pub trait ReportGeneratorRPC {
  async fn awards_report() -> Result<ReportData, String>;
  async fn teams_report() -> Result<ReportData, String>;
  async fn wpa_report(csv: bool) -> Result<ReportData, String>;
  async fn rankings_report() -> Result<ReportData, String>;
  async fn match_report(match_type: MatchType, per_team: bool) -> Result<ReportData, String>;
}