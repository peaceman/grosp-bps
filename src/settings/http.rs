use serde::Deserialize;
use std::net::{SocketAddr, IpAddr, Ipv6Addr};

use super::SettingsError;

#[derive(Debug)]
pub struct Http {
    pub socket: SocketAddr,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 2350),
        }
    }
}

impl Http {
    pub fn new(sources: Vec<PartialHttp>) -> Result<Self, SettingsError> {
        let merged: PartialHttp = sources.iter().fold(Default::default(), |acc, x| {
            PartialHttp {
                socket: acc.socket.or(x.socket),
            }
        });

        Ok(Http {
            socket: merged.socket.ok_or_else(|| SettingsError::MissingValue("http.socket".to_string()))?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialHttp {
    pub socket: Option<SocketAddr>,
}

impl Default for PartialHttp {
    fn default() -> Self {
        PartialHttp {
            socket: None
        }
    }
}
