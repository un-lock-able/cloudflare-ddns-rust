pub mod cmd;
pub mod file;

use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum RecordType {
    A,
    #[serde(rename = "AAAA")]
    Aaaa,
}

impl std::fmt::Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RecordType::A => write!(f, "A"),
            RecordType::Aaaa => write!(f, "AAAA"),
        }
    }
}

impl From<IpAddr> for RecordType {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(_) => Self::A,
            IpAddr::V6(_) => Self::Aaaa,
        }
    }
}
