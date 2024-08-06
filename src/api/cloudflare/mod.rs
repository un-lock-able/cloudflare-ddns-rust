mod api;

use api::{request, response};
use serde::Deserialize;
use std::collections::HashMap;

use super::ApiInterface;

use crate::api::RecordDetail;

#[derive(Deserialize, Debug, Clone)]
pub struct CloudflareInterfaceBuildConfig {
    zone_id: String,
    api_token: String,
}

// Currently, the build config is exactely the config needed to be stored.
type CloudflareInterfaceConfig = CloudflareInterfaceBuildConfig;

pub struct CloudflareInterface {
    config: CloudflareInterfaceConfig,
    reqwest_client: reqwest::blocking::Client,
    record_id_cache: HashMap<String, String>,
}

impl CloudflareInterface {
    fn get_full_domain(subdomain: &str, base_domain: &str) -> String {
        if subdomain == "@" || subdomain == "" {
            base_domain.to_string()
        } else {
            format!("{}.{}", subdomain, base_domain)
        }
    }
}

impl ApiInterface for CloudflareInterface {
    type BuildConfigType = CloudflareInterfaceBuildConfig;
    fn new(config: Self::BuildConfigType) -> Self {
        Self {
            config,
            reqwest_client: reqwest::blocking::Client::new(),
            record_id_cache: HashMap::new(),
        }
    }

    fn authorize(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn describe_record(
        &mut self,
        subdomain_name: &str,
        base_domain_name: &str,
        record_type: crate::RecordType,
    ) -> Result<Option<super::RecordDetail>, String> {
        log::trace!("Cloudflare api interface describe_record");
        let full_domain_name = Self::get_full_domain(subdomain_name, base_domain_name);
        log::trace!("Full domain name: {}", full_domain_name);

        // Create the url for send, including the get params
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.config.zone_id
        );
        let params = [
            ("name", full_domain_name.clone()),
            ("type", format!("{}", record_type)),
        ];
        let url = match reqwest::Url::parse_with_params(&url, params) {
            Ok(result) => result,
            Err(reason) => {
                return Err(format!("Error parsing url with params: {}", reason));
            }
        };

        log::trace!("Encoded url: {}", url);

        // Create the request with authorization
        let request = self
            .reqwest_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.config.api_token));

        // Get response and decode into struct
        log::trace!("Send describe request");
        let response = match request.send() {
            Ok(response) => response,
            Err(reason) => return Err(format!("Error sending https request: {}", reason)),
        };

        let response_text = match response.text() {
            Ok(content) => content,
            Err(reason) => return Err(format!("Error decoding response: {}", reason)),
        };

        log::trace!("Describe record returned text: {}", response_text);

        let record_detail: response::DescribeRecord = match serde_json::from_str(&response_text) {
            Ok(parsed_details) => parsed_details,
            Err(reason) => return Err(format!("Error parse json: {}", reason)),
        };

        if !record_detail.success {
            return Err(format!(
                "Describe record api failed: {}",
                record_detail
                    .errors
                    .iter()
                    .fold(String::new(), |prev_string, cur_pair| {
                        format!("{}[{}]", prev_string, cur_pair)
                    })
            ));
        }

        // Now the api request should be successful
        match record_detail.result.len() {
            0 => Ok(None),
            1 => {
                self.record_id_cache
                    .insert(full_domain_name, record_detail.result[0].id.clone());
                Ok(Some(RecordDetail {
                    subdomain_name: subdomain_name.to_string(),
                    base_domain_name: base_domain_name.to_string(),
                    ttl: record_detail.result[0].ttl,
                    proxied: record_detail.result[0].proxied,
                    ip_addr: match record_detail.result[0].content.parse() {
                        Ok(address) => address,
                        Err(reason) => {
                            return Err(format!(
                                "Pase api-returned address as IP address failed: {}",
                                reason
                            ))
                        }
                    },
                }))
            }
            _ => Err(format!(
                "More than 1 records exists for {} record of {}.",
                record_type, full_domain_name
            )),
        }
    }

    fn create_record(&self, record_content: RecordDetail) -> Result<(), String> {
        log::trace!("Cloudflare api create_record");
        let full_domain_name = Self::get_full_domain(
            &record_content.subdomain_name,
            &record_content.base_domain_name,
        );

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.config.zone_id
        );

        let post_body = request::CreateRecord {
            name: full_domain_name.clone(),
            content: record_content.ip_addr.to_string(),
            record_type: record_content.ip_addr.into(),
            proxied: record_content.proxied.unwrap_or(false),
            ttl: record_content.ttl,
        };

        let post_body = match serde_json::to_string(&post_body) {
            Ok(body_string) => body_string,
            Err(reason) => return Err(format!("Cannot serialize post body: {}", reason)),
        };

        log::trace!("Create record post body serialized text: {}", post_body);

        let request = self
            .reqwest_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .body(post_body);

        let response = match request.send() {
            Ok(response) => response,
            Err(reason) => return Err(format!("Failed to send create record request: {}", reason)),
        };

        let response_text = match response.text() {
            Ok(text) => text,
            Err(reason) => {
                return Err(format!(
                    "Failed to parse returned request as text: {}",
                    reason
                ))
            }
        };

        log::trace!("Create record returned text: {}", response_text);

        let create_result: response::CreateRecord = match serde_json::from_str(&response_text) {
            Ok(parsed_details) => parsed_details,
            Err(reason) => {
                return Err(format!(
                    "Failed to parse returned request as json: {}",
                    reason
                ))
            }
        };

        if !create_result.success {
            return Err(format!(
                "Create record api failed: {}",
                create_result
                    .errors
                    .iter()
                    .fold(String::new(), |prev_string, cur_pair| {
                        format!("{}[{}]", prev_string, cur_pair)
                    })
            ));
        }

        if full_domain_name != create_result.result.name {
            return Err(format!(
                "Api returned domain name {} mismaches the request domain name {}",
                create_result.result.name, full_domain_name
            ));
        }

        Ok(())
    }

    fn update_record(&self, record_content: RecordDetail) -> Result<(), String> {
        log::trace!("Cloudflare api update_record");

        // The cloudflare api needs record id to update
        let full_domain_name = Self::get_full_domain(
            &record_content.subdomain_name,
            &record_content.base_domain_name,
        );

        let record_id = match self.record_id_cache.get(&full_domain_name) {
            Some(id) => id,
            None => return Err(format!("No record id cache for {}", full_domain_name)),
        };

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            self.config.zone_id, record_id
        );

        let put_body = request::UpdateRecord {
            name: full_domain_name,
            content: record_content.ip_addr.to_string(),
            record_type: record_content.ip_addr.into(),
            proxied: record_content.proxied.unwrap_or(false),
            ttl: record_content.ttl,
        };

        let put_body = match serde_json::to_string(&put_body) {
            Ok(body_string) => body_string,
            Err(reason) => return Err(format!("Serialize put body failed: {}", reason)),
        };

        let request = self
            .reqwest_client
            .put(url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .body(put_body);

        let response = match request.send() {
            Ok(response) => response,
            Err(reason) => return Err(format!("Failed to send update record request: {}", reason)),
        };

        let response_text = match response.text() {
            Ok(content) => content,
            Err(reason) => return Err(format!("Error decoding response: {}", reason)),
        };

        log::trace!("Update record returned text: {}", response_text);

        let update_result: response::UpdateRecord = match serde_json::from_str(&response_text) {
            Ok(parsed_details) => parsed_details,
            Err(reason) => {
                return Err(format!(
                    "Failed to parse returned request as json: {}",
                    reason
                ))
            }
        };

        if !update_result.success {
            return Err(format!(
                "Update record api failed: {}",
                update_result
                    .errors
                    .iter()
                    .fold(String::new(), |prev_string, cur_pair| {
                        format!("{}[{}]", prev_string, cur_pair)
                    })
            ));
        }

        Ok(())
    }
}
