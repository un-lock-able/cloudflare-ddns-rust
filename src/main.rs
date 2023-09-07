mod cloudflare_api;
mod config_parser;
mod domain_record_changer;

use chrono::Utc;
use clap::Parser;
use domain_record_changer::DomainRecordChanger;
use log::LevelFilter;
use std::{fs::File, io::BufReader, thread};

use crate::domain_record_changer::RecordType;

fn main() {
    let args = config_parser::CmdArgs::parse();

    let mut log_file_path: std::path::PathBuf;

    match args.log_file {
        Some(path) => log_file_path = std::path::Path::new(&path).into(),
        None => {
            let exe_path = std::env::current_exe().unwrap();

            log_file_path = exe_path.clone();
            log_file_path.pop();
            log_file_path.push("ddnslog.log");
        }
    }

    let log_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_file_path)
        .unwrap_or_else(|reason| {
            panic! {"Open log file {} failed: {}", log_file_path.display(), reason}
        });

    if args.debug {
        // simple_logging::log_to_stderr(LevelFilter::Debug);
        simple_logging::log_to(log_file, LevelFilter::Debug)
    } else {
        // simple_logging::log_to_stderr(LevelFilter::Info);
        simple_logging::log_to(log_file, LevelFilter::Info)
    }

    log::info!(
        "DDNS script started at {}.",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    let settings_file = File::open(&args.config).unwrap_or_else(|reason| {
        log::error!("Open config file {} failed: {}", args.config, reason);
        panic!("Open config file {} failed: {}", args.config, reason);
    });

    let settings_file_reader = BufReader::new(settings_file);

    let settings: config_parser::DDNSSetings = serde_json::from_reader(settings_file_reader)
        .unwrap_or_else(|_| {
            log::error!("Parse config file failed");
            panic!("Parse config file faied");
        });

    let ipv4_address = thread::spawn(|| {
        let result = match reqwest::blocking::get(settings.get_ip_urls.ipv4) {
            Ok(content) => content,
            Err(reason) => {
                log::error!(
                    "Get IPv4 address failed: {}. Will not update any A record.",
                    reason
                );
                return Err(());
            }
        };
        let result = match result.text() {
            Ok(content) => content,
            Err(reason) => {
                log::error!(
                    "Get IPv4 address failed: {}. Will not update any A record.",
                    reason
                );
                return Err(());
            }
        };
        return Ok(result);
    });

    let ipv6_address = thread::spawn(|| {
        let result = match reqwest::blocking::get(settings.get_ip_urls.ipv6) {
            Ok(content) => content,
            Err(reason) => {
                log::error!(
                    "Get IPv6 address failed: {}. Will not update any AAAA record.",
                    reason
                );
                return Err(());
            }
        };
        let result = match result.text() {
            Ok(content) => content,
            Err(reason) => {
                log::error!(
                    "Get IPv6 address failed: {}. Will not update any AAAA record.",
                    reason
                );
                return Err(());
            }
        };
        return Ok(result);
    });

    let ipv4_address = ipv4_address.join().unwrap_or_else(|_| {
        log::error!("Get ipv4 address failed: thread exited abnormally.");
        return Err(());
    });

    if let Ok(address) = &ipv4_address {
        log::info!("Got ipv4 address: {}", address);
    }

    let ipv6_address = ipv6_address.join().unwrap_or_else(|_| {
        log::error!("Get ipv6 address failed: thread exited abnormally.");
        return Err(());
    });

    if let Ok(address) = &ipv6_address {
        log::info!("Got ipv6 address: {}", address);
    }

    let mut tasks = Vec::new();

    for single_domain_settings in settings.domain_settings {
        let current_ip_address: &String;
        match single_domain_settings.record_type {
            RecordType::A => match &ipv4_address {
                Ok(address) => current_ip_address = address,
                Err(_) => {
                    log::error!("Skipping A record update for {} as a result of previously failed ip address aquisition.", single_domain_settings.domain_name);
                    continue;
                }
            },
            RecordType::AAAA => match &ipv6_address {
                Ok(address) => current_ip_address = address,
                Err(_) => {
                    log::error!("Skipping AAAA record update for {} as a result of previously failed ip address aquisition.", single_domain_settings.domain_name);
                    continue;
                }
            },
        }
        let changer = DomainRecordChanger::new(single_domain_settings, current_ip_address.clone());
        tasks.push(thread::spawn(move || changer.start_ddns()));
    }

    log::trace!("Creating tasks success");

    for task in tasks {
        task.join()
            .unwrap_or_else(|_| log::error!("Async function error."));
    }

    log::info!("DDNS script ended.");
}
