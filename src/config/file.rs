use crate::{api::ServiceProvider, RecordType};
use serde::Deserialize;

pub enum FileType {
    Json,
    Toml,
}

#[derive(Debug, Deserialize)]
pub struct DDNSSetings {
    pub get_ip_urls: GetIPUrls,
    pub domain_settings: Vec<SingleDomainSettings>,
}

#[derive(Debug, Deserialize)]
pub struct GetIPUrls {
    pub ipv4: String,
    pub ipv6: String,
}

#[derive(Debug, Deserialize)]
pub struct SingleDomainSettings {
    pub enabled: bool,
    pub domain_name: String,
    pub service_provider: ServiceProvider,
    pub record_type: RecordType,
    #[serde(default = "default_create_new_record")]
    pub create_new_record: bool,
    pub subdomains: Vec<SubDomainSettings>,
}

fn default_create_new_record() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct SubDomainSettings {
    pub name: String,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
    pub proxied: Option<bool>,
    pub interface_id: Option<String>,
}

fn default_ttl() -> u32 {
    1
}
