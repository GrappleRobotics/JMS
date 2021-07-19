use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "service-configs"]
pub struct ServiceConfigs;