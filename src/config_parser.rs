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
    #[serde(rename = "interfaceID")]
    pub interface_id: Option<String>,
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
    #[arg(short, long, required = true)]
    pub config: String,
    #[arg(
        long,
        help = "Path to the log file. Will create all the parent directory if none exist. Defaults to ddnslog.log file in the same directory as the excutable."
    )]
    pub log_file: Option<String>,
    #[arg(
        short = 'n',
        help = "The number of threads used to update the domains.",
        default_value_t = 4
    )]
    pub thread_number: u8,
}
