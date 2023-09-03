mod config_parser;
use std::{fs::File, io::BufReader};

use chrono::Utc;
use clap::Parser;
use log::LevelFilter;

struct IpAddress {
    ipv4: String,
    ipv6: String,
}

fn main() {
    let args = config_parser::CmdArgs::parse();

    let mut exe_path = std::env::current_exe().unwrap();
    exe_path.pop();
    let mut log_path = exe_path.clone();
    log_path.push(Utc::now().format("%F-%H%M%S.log").to_string());

    if let Err(reason) = if args.debug {
        simple_logging::log_to_file(log_path, LevelFilter::Debug)
    } else {
        simple_logging::log_to_file(log_path, LevelFilter::Info)
    } {
        panic!("Open log file failed: {}", reason);
    }

    let settings_file = File::open(args.config).unwrap_or_else(|reason| {
        log::error!("Open config file failed: {}", reason);
        panic!("Open config file failed.");
    });

    let settings_file_reader = BufReader::new(settings_file);

    let settings: config_parser::DDNSSetings = serde_json::from_reader(settings_file_reader)
        .unwrap_or_else(|_| {
            log::error!("Parse config file failed");
            panic!("Parse config file faied");
        });

    let ipv4_address = reqwest::blocking::get(settings.get_ip_urls.ipv4)
        .unwrap_or_else(|reason| {
            log::error!("Get IPv4 address failed: {}", reason);
            panic!("Get IPv4 address failed: {}", reason)
        })
        .text()
        .unwrap_or_else(|reason| {
            log::error!("Get IPv4 address failed: {}", reason);
            panic!("Get IPv4 address failed: {}", reason)
        });

    let ipv6_address = reqwest::blocking::get(settings.get_ip_urls.ipv6)
        .unwrap_or_else(|reason| {
            log::error!("Get IPv4 address failed: {}", reason);
            panic!("Get IPv4 address failed: {}", reason)
        })
        .text()
        .unwrap_or_else(|reason| {
            log::error!("Get IPv4 address failed: {}", reason);
            panic!("Get IPv4 address failed: {}", reason)
        });

    println!("{}\r\n{}", ipv4_address, ipv6_address);
}
