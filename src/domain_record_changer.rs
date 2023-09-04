use crate::cloudflare_api;
use crate::config_parser::SingleDomainSettings;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct IpAddress {
    pub ipv4: String,
    pub ipv6: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RecordType {
    A,
    AAAA,
}

pub struct DomainRecordChanger {
    settings: SingleDomainSettings,
    ip_address: IpAddress,
    reqwest_client: reqwest::blocking::Client,
}

impl DomainRecordChanger {
    pub fn new(settings: SingleDomainSettings, ip_address: IpAddress) -> Self {
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
        log::debug!("Get detail infomation for {}", full_domain_name);
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
                    "Parse url param error when getting detail information for {}: {}",
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
                            "Parse api-returned record detail for {} failed: {}",
                            full_domain_name,
                            reason
                        );
                        return Err(());
                    }
                }
            }
            Err(reason) => {
                log::error!("Get detail for {} error: {}", full_domain_name, reason);
                return Err(());
            }
        };
        return Ok(record_status);
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
            }

            if record_detail.result_info.count == 0 {
                if self.settings.create_new_record {
                    log::info!(
                        "{} record for {} does not exist. Will create new record.",
                        self.settings.record_type,
                        full_domain_name
                    );
                } else {
                    log::info!(
                        "{} record for {} does not exist. Will not create new record.",
                        self.settings.record_type,
                        full_domain_name
                    )
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
