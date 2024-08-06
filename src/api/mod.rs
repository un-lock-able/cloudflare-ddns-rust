pub mod cloudflare;

use crate::RecordType;
use core::net::IpAddr;
use serde::Deserialize;

/// A enum representing all supported service provider and their build config. Should be deserialized from
/// config file.
#[derive(Deserialize, Clone)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum ServiceProvider {
    Cloudflare(cloudflare::CloudflareInterfaceBuildConfig),
}

pub struct RecordDetail {
    pub subdomain_name: String,
    pub base_domain_name: String,
    pub ttl: u32,
    // This is currently only for cloudflare.
    pub proxied: Option<bool>,
    pub ip_addr: IpAddr,
}

// Impl this ourselves so that we can let the ddns changer consider there's no need to change when the api returned
// record detail's proxied is None.
impl PartialEq for RecordDetail {
    fn eq(&self, other: &Self) -> bool {
        let subdomain_name_eq = self.subdomain_name == other.subdomain_name;
        let base_domain_name_eq = self.base_domain_name == other.base_domain_name;
        let ttl_eq = self.ttl == other.ttl;
        let proxied_eq = self.proxied.is_none() || other.proxied.is_none() || self.proxied == other.proxied;
        let ip_addr_eq = self.ip_addr == other.ip_addr;
        subdomain_name_eq && base_domain_name_eq && ttl_eq && proxied_eq && ip_addr_eq
    }
}

/// A trait for the api interface used in ddns changer.
///
/// Now, it is garanteed that whenever an instance of `ApiInterface`
/// is created, `authorize` is first called. Then, for every distinct
/// domain name (subdomain_name + domain_name), `describe_record`
/// is called before either `create_record` or `update_record` is called,
/// so if `create_record` or `update_record` needs something like "record_id" or other api-specific information,
/// these infomation can be stored in the instance when `describe_record` is called.
pub trait ApiInterface {
    /// The Config to build the ApiInterface from. This should be the struct in the config file.
    type BuildConfigType: Clone + for<'a> Deserialize<'a>;

    /// Returns a new interface instance.
    fn new(config: Self::BuildConfigType) -> Self;

    /// Authorize self to the api. If failed, return a string specify the reason.
    fn authorize(&mut self) -> Result<(), String>;

    /// Return the detail of a record from api. If the record doesn't exist, return
    /// Ok(None). If the api didn't success, return a String containing fail reason.
    ///
    /// Note: having multiple record for one domain is also considered an error.
    ///
    /// It is the interface's responsibility to store additional infomation that may be needed
    /// for create or update the record.
    fn describe_record(
        &mut self,
        subdomain_name: &str,
        base_domain_name: &str,
        record_type: RecordType,
    ) -> Result<Option<RecordDetail>, String>;

    /// Create a record of `subdomain_name`.`base_domain_name` with `record_content`. Only called when
    /// `describe_record` returned `Some(None)`, and user sets to create on none-exsistent domains.
    /// The type of the record can be inferred from `IpAddr` struct.
    ///
    /// On success, return `Ok()`; on failure, return `Err` containing reason string.
    fn create_record(&self, record_detail: RecordDetail) -> Result<(), String>;

    /// Change the content of `subdomain_name`.`base_domain_name` to `record_content`. Only called when
    /// `describe_record` reports that exactely one domain exists. The type of the record can be inferred
    /// from `IpAddr` struct.
    ///
    /// On success, return `Ok()`; on failure, return `Err` containing reason string.
    fn update_record(&self, record_detail: RecordDetail) -> Result<(), String>;
}
