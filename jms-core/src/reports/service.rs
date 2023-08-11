use jms_base::{kv::KVConnection, mq::MessageQueueChannel};
use jms_core_lib::{reports::{ReportGeneratorRPC, ReportData}, models::MatchType};

use super::{award_report, team_report, rankings_report, wpa_key_report, match_report};

pub struct ReportGeneratorService {
  pub kv: KVConnection,
  pub mq: MessageQueueChannel
}

#[async_trait::async_trait]
impl ReportGeneratorRPC for ReportGeneratorService {
  fn mq(&self) -> &MessageQueueChannel { &self.mq }

  async fn awards_report(&mut self) -> Result<ReportData, String> {
    award_report::awards_report(&self.kv).map_err(|e| e.to_string())
  }

  async fn teams_report(&mut self) -> Result<ReportData, String> {
    team_report::teams_report(&self.kv).map_err(|e| e.to_string())
  }

  async fn wpa_report(&mut self, csv: bool) -> Result<ReportData, String> {
    if csv {
      wpa_key_report::wpa_report_csv(&self.kv).map_err(|e| e.to_string())
    } else {
      wpa_key_report::wpa_report(&self.kv).map_err(|e| e.to_string())
    }
  }

  async fn rankings_report(&mut self) -> Result<ReportData, String> {
    rankings_report::rankings_report(&self.kv).map_err(|e| e.to_string())
  }

  async fn match_report(&mut self, match_type: MatchType, per_team: bool) -> Result<ReportData, String> {
    if per_team {
      match_report::match_report_per_team(match_type, &self.kv).map_err(|e| e.to_string())
    } else {
      match_report::match_report(match_type, &self.kv).map_err(|e| e.to_string())
    }
  }
}

impl ReportGeneratorService {
  pub async fn run(&mut self) -> anyhow::Result<()> {
    let mut rpc = self.rpc_handle().await?;
    loop {
      self.rpc_process(rpc.next().await).await?;
    }
  }
}