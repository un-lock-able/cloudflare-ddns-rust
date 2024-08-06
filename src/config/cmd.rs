use clap::{Parser, ValueEnum};
use log::LevelFilter;

#[derive(ValueEnum, Clone)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trace => write!(f, "trace"),
            Self::Debug => write!(f, "debug"),
            Self::Info => write!(f, "info"),
            Self::Warn => write!(f, "warn"),
            Self::Error => write!(f, "error"),
        }
    }
}

impl std::convert::Into<LevelFilter> for LogLevel {
    fn into(self) -> LevelFilter {
        match self {
            Self::Trace => LevelFilter::Trace,
            Self::Debug => LevelFilter::Debug,
            Self::Info => LevelFilter::Info,
            Self::Warn => LevelFilter::Warn,
            Self::Error => LevelFilter::Error,
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CmdArgs {
    #[arg(short, long, required = true)]
    pub config: String,
    #[arg(
        long,
        help = "Write log to file. Will create all parent folder if not exist."
    )]
    pub log_file: Option<String>,
    #[arg(long, default_value_t = LogLevel::Info, help = "Specify the log level.")]
    pub log_level: LogLevel,
    #[arg(
        short = 'n',
        help = "The number of threads used to update the domains.",
        default_value_t = 4
    )]
    pub thread_number: u8,
}
