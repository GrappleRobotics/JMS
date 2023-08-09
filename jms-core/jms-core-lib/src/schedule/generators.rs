#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct QualsMatchGeneratorParams {
  pub team_anneal_steps: usize,
  pub station_anneal_steps: usize,
}

pub const MATCH_GENERATOR_JOB_KEY: &'static str = "job:match_gen:working";

#[jms_macros::service]
pub trait MatchGeneratorRPC {
  async fn start_qual_gen(params: QualsMatchGeneratorParams) -> Result<(), String>;
  async fn reset_playoffs() -> Result<(), String>;
  async fn update_playoffs() -> Result<(), String>;
}