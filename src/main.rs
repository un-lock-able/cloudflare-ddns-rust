mod cloudflare_api;
mod config_parser;
mod domain_record_changer;

use chrono::Utc;
use clap::Parser;
use domain_record_changer::{DomainRecordChanger, IpAddress};
use log::LevelFilter;
use std::{fs::File, io::BufReader, thread};

fn main() {
    let args = config_parser::CmdArgs::parse();

    let mut log_path: std::path::PathBuf;
    
    match args.log_dir {
        Some(path) => log_path = std::path::Path::new(&path).into(),
        None => {
            let mut exe_path = std::env::current_exe().unwrap();
            exe_path.pop();
        
            log_path = exe_path.clone();
            log_path.push("logs");
        }
    }
    
    if !log_path.exists() {
        if let Err(reason) =  std::fs::create_dir_all(&log_path) {
            eprintln!("Error creating log file path {}: {}", log_path.display(), reason);
            panic!("Error creating log file path {}: {}", log_path.display(), reason);
        }
    }
    log_path.push(Utc::now().format("%F-%H%M%S.log").to_string());

    if let Err(reason) = if args.debug {
        // simple_logging::log_to_stderr(LevelFilter::Debug);
        simple_logging::log_to_file(log_path, LevelFilter::Debug)
    } else {
        // simple_logging::log_to_stderr(LevelFilter::Info);
        simple_logging::log_to_file(log_path, LevelFilter::Info)
    } {
        panic!("Open log file failed: {}", reason);
    }

    log::info!("DDNS script started.");

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
        reqwest::blocking::get(settings.get_ip_urls.ipv4)
            .unwrap_or_else(|reason| {
                log::error!("Get IPv4 address failed: {}", reason);
                panic!("Get IPv4 address failed: {}", reason)
            })
            .text()
            .unwrap_or_else(|reason| {
                log::error!("Get IPv4 address failed: {}", reason);
                panic!("Get IPv4 address failed: {}", reason)
            })
    });

    

    let ipv6_address = thread::spawn(|| {
        reqwest::blocking::get(settings.get_ip_urls.ipv6)
            .unwrap_or_else(|reason| {
                log::error!("Get IPv6 address failed: {}", reason);
                panic!("Get IPv6 address failed: {}", reason)
            })
            .text()
            .unwrap_or_else(|reason| {
                log::error!("Get IPv6 address failed: {}", reason);
                panic!("Get IPv6 address failed: {}", reason)
            })
    });

    let ipv4_address = ipv4_address.join()
    .unwrap_or_else(|_| {
        log::error!("Get ipv4 address failed.");
        panic!("Get ipv4 address failed.")
    });

    log::info!("Got ipv4 address: {}", ipv4_address);

    let ipv6_address = ipv6_address
    .join()
    .unwrap_or_else(|_| {
        log::error!("Get ipv6 address failed.");
        panic!("Get ipv6 address failed.")
    });

    log::info!("Got ipv6 address: {}", ipv6_address);

    let ip_addresses = IpAddress {
        ipv4: ipv4_address,
        ipv6: ipv6_address,
    };

    let mut tasks = Vec::new();

    log::trace!("Start creating tasks.");

    for single_domain_settings in settings.domain_settings {
        log::trace!("Create changer for {}", single_domain_settings.domain_name);
        let changer = DomainRecordChanger::new(single_domain_settings, ip_addresses.clone());
        tasks.push(thread::spawn(move || {changer.start_ddns()}));
    }

    log::trace!("Creating tasks success");

    for task in tasks {
        task.join()
            .unwrap_or_else(|_| log::error!("Async function error."));
    }

    log::info!("DDNS script ended.");
}
