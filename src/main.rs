mod cloudflare_api;
mod config_parser;
mod domain_record_changer;

use chrono::Utc;
use clap::Parser;
use domain_record_changer::DomainRecordChanger;
use log::LevelFilter;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::{fs::File, io::BufReader, thread};

use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};

use crate::domain_record_changer::RecordType;

fn main() {
    let args = config_parser::CmdArgs::parse();

    let stderr = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S %Z)(utc)}] {i} {h({l})} {m}\n",
        )))
        .build();

    let mut config_builder = Config::builder().appender(
        Appender::builder()
            .filter(Box::new(ThresholdFilter::new(
                args.log_level.clone().into(),
            )))
            .build("stderr", Box::new(stderr)),
    );

    let mut root_builder = Root::builder().appender("stderr");

    if let Some(log_file_name) = args.log_file {
        let log_file_appender = FileAppender::builder()
            // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
            .encoder(Box::new(PatternEncoder::new(
                "[{d(%Y-%m-%d %H:%M:%S %Z)(utc)}] {i} {h({l})} {m}\n",
            )))
            .build(&log_file_name)
            .unwrap_or_else(|reason| {
                panic! {"Open log file {} failed: {}", log_file_name, reason}
            });
        config_builder = config_builder.appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(
                    args.log_level.clone().into(),
                )))
                .build("logfile", Box::new(log_file_appender)),
        );
        root_builder = root_builder.appender("logfile");
    }
    
    let log_root = root_builder.build(LevelFilter::Trace);

    let config = config_builder
        .build(log_root)
        .unwrap_or_else(|reason| panic!("Setup log settings filed: {}", reason));

    let _log_handle = log4rs::init_config(config)
        .unwrap_or_else(|reason| panic!("Setup log settings filed: {}", reason));

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

    let ipv6_address = ipv6_address.join().unwrap_or_else(|_| {
        log::error!("Get ipv6 address failed: thread exited abnormally.");
        return Err(());
    });

    let ipv4_address = match ipv4_address {
        Ok(content) => {
            let content_ip = content.parse::<Ipv4Addr>();
            match content_ip {
                Ok(address) => {
                    log::info!("Got ipv4 addres: {}", address.to_string());
                    Ok(address)
                }
                Err(_) => {
                    log::error!(
                        "Content returned from the ipv4 api cannot be parsed as an IPv4 address. Content: {}", content
                    );
                    Err(())
                }
            }
        }
        Err(_) => Err(()),
    };

    let ipv6_address = match ipv6_address {
        Ok(content) => {
            let content_ip = content.parse::<Ipv6Addr>();
            match content_ip {
                Ok(address) => {
                    log::info!("Got ipv6 address: {}", address.to_string());
                    Ok(address)
                }
                Err(_) => {
                    log::error!(
                        "Content returned from the ipv6 api cannot be parsed as an IPv4 address. Content: {}", content
                    );
                    Err(())
                }
            }
        }
        Err(_) => Err(()),
    };

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.thread_number.into())
        .build()
        .unwrap_or_else(|reason| {
            log::error!("Create thread pool failed: {}", reason);
            panic!("Create thread pool failed: {}", reason)
        });

    pool.scope(|s| {
    for single_domain_settings in settings.domain_settings {
        let current_ip_address: IpAddr;
        match single_domain_settings.record_type {
            RecordType::A => match &ipv4_address {
                Ok(address) => current_ip_address = IpAddr::V4(*address),
                Err(_) => {
                    log::error!("Skipping A record update for {} as a result of previously failed ip address aquisition.", single_domain_settings.domain_name);
                    continue;
                }
            },
            RecordType::AAAA => match &ipv6_address {
                Ok(address) => current_ip_address = IpAddr::V6(*address),
                Err(_) => {
                    log::error!("Skipping AAAA record update for {} as a result of previously failed ip address aquisition.", single_domain_settings.domain_name);
                    continue;
                }
            },
        }
        let changer = DomainRecordChanger::new(single_domain_settings, current_ip_address.clone());
        s.spawn(|_| {changer.start_ddns();});
    }
});

    log::info!("DDNS script ended.");
}
