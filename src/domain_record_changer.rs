use crate::config_parser::SingleDomainSettings;
use crate::{cloudflare_api, config_parser::SubDomainSettings};
use serde::{Deserialize, Serialize};
use std::net::{Ipv6Addr, IpAddr};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum RecordType {
    A,
    AAAA,
}

pub struct DomainRecordChanger {
    settings: SingleDomainSettings,
    ip_address: IpAddr,
    reqwest_client: reqwest::blocking::Client,
}

impl DomainRecordChanger {
    pub fn new(settings: SingleDomainSettings, ip_address: IpAddr) -> Self {
        DomainRecordChanger {
            settings,
            ip_address,
            reqwest_client: reqwest::blocking::Client::new(),
        }
    }

    fn get_record_status(
        &self,
        full_domain_name: &String,
    ) -> Result<cloudflare_api::response::DescribeRecord, ()> {
        log::debug!(
            "Get detail infomation for {} record of {}",
            self.settings.record_type,
            full_domain_name
        );
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.settings.zone_id
        );
        let params = [
            ("name", full_domain_name.clone()),
            ("type", format!("{}", self.settings.record_type)),
        ];
        let url = match reqwest::Url::parse_with_params(&url, params) {
            Ok(result) => result,
            Err(reason) => {
                log::error!(
                    "Parse url param error when getting detail information for {} record of {}: {}",
                    self.settings.record_type,
                    full_domain_name,
                    reason
                );
                unreachable!("Should never parse reqwest param error.")
            }
        };

        let request = self.reqwest_client.get(url).header(
            "Authorization",
            format!("Bearer {}", self.settings.api_token),
        );

        let record_status: cloudflare_api::response::DescribeRecord = match request.send() {
            Ok(result) => {
                let result = result.text().unwrap_or(String::from(""));
                match serde_json::from_str(&result) {
                    Ok(parsed_details) => parsed_details,
                    Err(reason) => {
                        log::error!(
                            "Parse api-returned record detail for {} record of {} failed: {}.",
                            self.settings.record_type,
                            full_domain_name,
                            reason
                        );
                        log::debug!("Returned string: {}", result);
                        return Err(());
                    }
                }
            }
            Err(reason) => {
                log::error!(
                    "Get detail for {} record of {} error: {}",
                    self.settings.record_type,
                    full_domain_name,
                    reason
                );
                return Err(());
            }
        };
        return Ok(record_status);
    }

    fn create_new_record(
        &self,
        full_domain_name: &String,
        subdomain_setting: &SubDomainSettings,
        content_ip_address: &String,
    ) -> Result<cloudflare_api::response::CreateRecord, ()> {
        log::debug!(
            "Create {} record for {}",
            self.settings.record_type,
            full_domain_name
        );

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.settings.zone_id
        );

        let post_body = cloudflare_api::request::CreateRecord {
            name: full_domain_name.clone(),
            content: content_ip_address.clone(),
            record_type: self.settings.record_type,
            proxied: subdomain_setting.proxied,
            ttl: subdomain_setting.ttl,
        };

        let post_body = match serde_json::to_string(&post_body) {
            Ok(body_string) => body_string,
            Err(reason) => {
                log::error!(
                    "Cannot serialize post body for creating {} record for {}: {}",
                    self.settings.record_type,
                    full_domain_name,
                    reason
                );
                return Err(());
            }
        };

        let request = self
            .reqwest_client
            .post(url)
            .header(
                "Authorization",
                format!("Bearer {}", self.settings.api_token),
            )
            .header("Content-Type", "application/json")
            .body(post_body);

        let create_result: cloudflare_api::response::CreateRecord = match request.send() {
            Ok(result) => {
                let result = result.text().unwrap_or(String::from(""));
                match serde_json::from_str(&result) {
                    Ok(parsed_details) => parsed_details,
                    Err(reason) => {
                        log::error!(
                            "Parse api-returned create record result for {} record of {} failed: {}",
                            self.settings.record_type,
                            full_domain_name,
                            reason,
                        );
                        log::debug!("Returned string: {}", result);
                        return Err(());
                    }
                }
            }
            Err(reason) => {
                log::error!(
                    "Get create record result for {} record of {} error: {}",
                    self.settings.record_type,
                    full_domain_name,
                    reason
                );
                return Err(());
            }
        };
        return Ok(create_result);
    }

    fn update_record(
        &self,
        full_domain_name: &String,
        subdomain_setting: &SubDomainSettings,
        target_record_id: &String,
        content_ip_address: &String,
    ) -> Result<cloudflare_api::response::UpdateRecord, ()> {
        log::debug!(
            "Update {} record for {}",
            self.settings.record_type,
            full_domain_name
        );

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            self.settings.zone_id, target_record_id
        );

        let put_body = cloudflare_api::request::UpdateRecord {
            name: full_domain_name.clone(),
            content: content_ip_address.clone(),
            record_type: self.settings.record_type,
            proxied: subdomain_setting.proxied,
            ttl: subdomain_setting.ttl,
        };

        let put_body = match serde_json::to_string(&put_body) {
            Ok(body_string) => body_string,
            Err(reason) => {
                log::error!(
                    "Cannot serialize put body for updating {} record for {}: {}",
                    self.settings.record_type,
                    full_domain_name,
                    reason
                );
                return Err(());
            }
        };

        let request = self
            .reqwest_client
            .put(url)
            .header(
                "Authorization",
                format!("Bearer {}", self.settings.api_token),
            )
            .header("Content-Type", "application/json")
            .body(put_body);

        let update_result: cloudflare_api::response::UpdateRecord = match request.send() {
            Ok(result) => {
                let result = result.text().unwrap_or(String::from(""));
                match serde_json::from_str(&result) {
                    Ok(parsed_details) => parsed_details,
                    Err(reason) => {
                        log::error!(
                            "Parse api-returned update record result for {} record of {} failed: {}",self.settings.record_type,
                            full_domain_name,
                            reason,
                        );
                        log::debug!("Returned string: {}", result);
                        return Err(());
                    }
                }
            }
            Err(reason) => {
                log::error!(
                    "Get update record result for {} record of {} error: {}",
                    self.settings.record_type,
                    full_domain_name,
                    reason,
                );
                return Err(());
            }
        };
        return Ok(update_result);
    }

    pub fn start_ddns(self) {
        log::debug!(
            "DDNS for {} of record type {} started",
            self.settings.domain_name,
            self.settings.record_type
        );

        if !self.settings.enabled {
            log::debug!(
                "DDNS for {} of record type {} is not enabled. Skipping.",
                self.settings.domain_name,
                self.settings.record_type
            );
            return;
        }

        'subdomain_iter: for subdomain_settings in &self.settings.subdomains {
            let full_domain_name: String;
            
            if subdomain_settings.name == "@" || subdomain_settings.name == "" {
                full_domain_name = self.settings.domain_name.clone();
            } else {
                full_domain_name =
                    format!("{}.{}", subdomain_settings.name, &self.settings.domain_name);
            }
            log::debug!("Start DDNS for {}", full_domain_name);

            // Convert the ip address to string to be sent by api request.
            let content_ip = match self.ip_address {
                IpAddr::V4(v4_address) => v4_address.to_string(), 
                IpAddr::V6(mut v6_address) => {
                    if let Some(custom_interface_id) = &subdomain_settings.interface_id {
                        log::debug!("The custom interface id for {} exists: {}", full_domain_name, custom_interface_id);
                        let mut v6_address_arr = v6_address.segments();

                        let interface_addr = custom_interface_id.parse::<Ipv6Addr>();
                        if interface_addr.is_err() {
                            log::error!("The custom interface id for {} cannot be parsed into a valid ipv6 addr. Settings string: {}.", full_domain_name, custom_interface_id);
                            continue 'subdomain_iter;
                        }
                        // Now interface_addr is definitely Ok()
                        let interface_addr = interface_addr.unwrap();
                        let interface_addr_arr = interface_addr.segments();

                        // Check if the first 64 bit is 0.
                        if interface_addr_arr.iter().take(4).fold(0, |accu, cur| accu + cur) != 0 {
                            log::warn!("The first 64 bit of the interface id is not 0. They are ignored.");
                        }
                        
                        // Copy the last 8 btyes into v6_address_arr and compose a new v6 address.
                        v6_address_arr[4..8].copy_from_slice(&interface_addr_arr[4..8]);
                        v6_address = Ipv6Addr::from(v6_address_arr);
                        log::debug!("The ip for {} will be {}, different from current machine's ip.", full_domain_name, v6_address.to_string());
                    }
                    v6_address.to_string()
                }
            };


            // Get the information
            let record_detail = match self.get_record_status(&full_domain_name) {
                Ok(result) => result,
                Err(_) => continue 'subdomain_iter,
            };

            if !record_detail.success {
                log::error!(
                    "Get record detail for domain {} faild. Reason:",
                    full_domain_name
                );
                for code_message_pair in record_detail.errors {
                    log::error!("{}", code_message_pair);
                }
                continue 'subdomain_iter;
            }

            // If not record presents, create one (or not).
            if record_detail.result_info.count <= 0 {
                if self.settings.create_new_record {
                    log::info!(
                        "{} record for {} does not exist. Will create new record.",
                        self.settings.record_type,
                        full_domain_name
                    );
                    let create_result =
                        match self.create_new_record(&full_domain_name, &subdomain_settings, &content_ip) {
                            Ok(result) => result,
                            Err(_) => continue 'subdomain_iter,
                        };
                    if create_result.success {
                        log::info!(
                            "Created new {} record of {}.",
                            self.settings.record_type,
                            full_domain_name
                        );
                    } else {
                        log::error!(
                            "Create new {} record for {} failed. Reason:",
                            self.settings.record_type,
                            full_domain_name
                        );
                        for code_message_pair in create_result.errors {
                            log::error!("{}", code_message_pair);
                        }
                    }
                } else {
                    log::info!(
                        "{} record for {} does not exist. Will not create new record.",
                        self.settings.record_type,
                        full_domain_name
                    )
                }
                continue 'subdomain_iter;
            } else if record_detail.result_info.count > 1 {
                log::warn!(
                    "Multiple {} record for {} exsist. Will skip this comain.",
                    self.settings.record_type,
                    full_domain_name
                );
                continue 'subdomain_iter;
            } else if record_detail.result.len() != 1 {
                log::error!("Api-returned record count is 1 but there isn't exactely 1 record in the results[] array.");
                continue 'subdomain_iter;
            }

            // There already exsists exactely one record. Check if the content mathces current ip, if not, update it.

            if self.ip_address.to_string() == record_detail.result[0].content
                && subdomain_settings.ttl == record_detail.result[0].ttl
                && subdomain_settings.proxied == record_detail.result[0].proxied
            {
                log::debug!(
                    "{} record for {} is same as current ip.",
                    self.settings.record_type,
                    full_domain_name
                );
                continue 'subdomain_iter;
            }

            let target_record_id = record_detail.result[0].id.clone();

            let update_result =
                match self.update_record(&full_domain_name, &subdomain_settings, &target_record_id, &content_ip)
                {
                    Ok(result) => result,
                    Err(_) => continue 'subdomain_iter,
                };

            if update_result.success {
                log::info!(
                    "Updated {} record of {}.",
                    self.settings.record_type,
                    full_domain_name
                );
            } else {
                log::error!(
                    "Update {} record of {} failed. Reason:",
                    self.settings.record_type,
                    full_domain_name
                );
                for code_message_pair in update_result.errors {
                    log::error!("{}", code_message_pair);
                }
            }
        }
    }
}

impl std::fmt::Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RecordType::A => write!(f, "{}", "A"),
            RecordType::AAAA => write!(f, "{}", "AAAA"),
        }
    }
}
