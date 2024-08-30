use std::net::{IpAddr, Ipv6Addr};

use crate::{
    api::{ApiInterface, RecordDetail},
    config::{
        file::{SingleDomainSettings, SubDomainSettings},
        RecordType,
    },
};

pub struct DomainRecordChanger<S>
where
    S: ApiInterface,
{
    enabled: bool,
    domain_name: String,
    record_type: RecordType,
    create_new_record: bool,
    subdomains: Vec<SubDomainSettings>,
    ip_address: IpAddr,
    api_interface: S,
}

impl<S> DomainRecordChanger<S>
where
    S: ApiInterface,
{
    pub fn new(settings: SingleDomainSettings, ip_address: IpAddr, api_interface: S) -> Self {
        DomainRecordChanger {
            enabled: settings.enabled,
            domain_name: settings.domain_name,
            record_type: settings.record_type,
            create_new_record: settings.create_new_record,
            subdomains: settings.subdomains,
            ip_address,
            api_interface,
        }
    }

    pub fn start_ddns(&mut self) {
        log::debug!(
            "DDNS for {} of record type {} started",
            self.domain_name,
            self.record_type
        );

        if !self.enabled {
            log::debug!(
                "DDNS for {} of record type {} is not enabled. Skipping.",
                self.domain_name,
                self.record_type
            );
            return;
        }

        log::debug!("Authorizing api");
        match self.api_interface.authorize() {
            Ok(_) => log::debug!("Authorize api succeeded."),
            Err(message) => {
                log::error!("Authorize api failed: {}", message);
                return;
            }
        }

        'subdomain_iter: for subdomain_settings in &self.subdomains {
            let full_domain_name =
                if subdomain_settings.name == "@" || subdomain_settings.name.is_empty() {
                    self.domain_name.clone()
                } else {
                    format!("{}.{}", subdomain_settings.name, &self.domain_name)
                };
            log::debug!("Start DDNS for {}", full_domain_name);

            // Convert the ip address to ip to be sent by api request.
            let content_ip: IpAddr = match self.ip_address {
                IpAddr::V4(v4_address) => v4_address.into(),
                IpAddr::V6(mut v6_address) => {
                    if let Some(custom_interface_id) = &subdomain_settings.interface_id {
                        log::debug!(
                            "The custom interface id for {} exists: {}",
                            full_domain_name,
                            custom_interface_id
                        );
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
                        if interface_addr_arr.iter().take(4).sum::<u16>() != 0 {
                            log::warn!(
                                "The first 64 bits of the interface id are not 0. They are ignored."
                            );
                        }

                        // Copy the last 8 bytes into v6_address_arr and compose a new v6 address.
                        v6_address_arr[4..8].copy_from_slice(&interface_addr_arr[4..8]);
                        v6_address = Ipv6Addr::from(v6_address_arr);
                        log::debug!(
                            "The ip for {} will be {}, different from current machine's ip.",
                            full_domain_name,
                            v6_address.to_string()
                        );
                    }
                    v6_address.into()
                }
            };

            // Get the information
            let record_detail = match self.api_interface.describe_record(
                &subdomain_settings.name,
                &self.domain_name,
                self.ip_address.into(),
            ) {
                Ok(result) => result,
                Err(message) => {
                    log::error!(
                        "Describe record for {} record of {} failed: {}",
                        self.record_type,
                        full_domain_name,
                        message
                    );
                    continue 'subdomain_iter;
                }
            };

            // The record should be like this
            let target_record = RecordDetail {
                subdomain_name: subdomain_settings.name.clone(),
                base_domain_name: self.domain_name.clone(),
                ttl: subdomain_settings.ttl,
                proxied: subdomain_settings.proxied,
                ip_addr: content_ip,
            };

            // If no record presents, create one (or not).
            if record_detail.is_none() {
                if self.create_new_record {
                    log::info!(
                        "{} record for {} does not exist. Will create new record.",
                        self.record_type,
                        full_domain_name
                    );
                    match self.api_interface.create_record(target_record) {
                        Ok(_) => log::info!(
                            "Created new {} record of {}.",
                            self.record_type,
                            full_domain_name
                        ),
                        Err(message) => {
                            log::error!(
                                "Create new {} record of {} failed: {}",
                                self.record_type,
                                full_domain_name,
                                message
                            )
                        }
                    }
                } else {
                    log::info!(
                        "{} record for {} does not exist. Will not create new record.",
                        self.record_type,
                        full_domain_name
                    )
                }
                continue 'subdomain_iter;
            }

            let record_detail = record_detail.expect("Already dealt with is none");
            if record_detail == target_record {
                log::debug!(
                    "{} record for {} is same as current ip.",
                    self.record_type,
                    full_domain_name
                );
                continue 'subdomain_iter;
            }

            match self.api_interface.update_record(target_record) {
                Ok(_) => {
                    log::info!(
                        "Updated {} record of {}.",
                        self.record_type,
                        full_domain_name
                    );
                }
                Err(message) => {
                    log::error!(
                        "Update {} record of {} failed: {}",
                        self.record_type,
                        full_domain_name,
                        message
                    );
                    continue 'subdomain_iter;
                }
            };
        }
    }
}
