use crate::domain_record_changer::RecordType;
use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DDNSSetings {
    #[serde(rename = "getIPUrls")]
    pub get_ip_urls: GetIPUrls,
    #[serde(rename = "domainSettings")]
    pub domain_settings: Vec<SingleDomainSettings>,
}

#[derive(Deserialize)]
pub struct GetIPUrls {
    #[serde(rename = "IPv4")]
    pub ipv4: String,
    #[serde(rename = "IPv6")]
    pub ipv6: String,
}

#[derive(Deserialize)]
pub struct SingleDomainSettings {
    pub enabled: bool,
    #[serde(rename = "zoneID")]
    pub zone_id: String,
    #[serde(rename = "apiToken")]
    pub api_token: String,
    #[serde(rename = "recordType")]
    pub record_type: RecordType,
    #[serde(rename = "createNewRecord")]
    pub create_new_record: bool,
    #[serde(rename = "domainName")]
    pub domain_name: String,
    pub subdomains: Vec<SubDomainSettings>,
}

#[derive(Deserialize)]
pub struct SubDomainSettings {
    pub name: String,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
    #[serde(default = "default_proxied_choice")]
    pub proxied: bool,
}

fn default_proxied_choice() -> bool {
    true
}

fn default_ttl() -> u32 {
    1
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CmdArgs {
    #[arg(long, default_value_t = false)]
    pub debug: bool,
    #[arg(short, long, default_value_t = String::from("cloudflareddnsSettings.json"))]
    pub config: String,
}
