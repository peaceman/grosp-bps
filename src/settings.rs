mod consul;
mod error;
mod http;
mod playlist;
mod segment_signing;

use consul::*;
use error::*;
use http::*;
use playlist::*;
use segment_signing::*;

use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::result::Result;
use std::time::Duration;

use serde::Deserialize;

const DEFAULT_CONSUL_UPDATE_INTERVAL_SECS: u64 = 1;
const DEFAULT_PLAYLIST_SEGMENT_SIGNING_DURATION_SECS: u64 = 60;

#[derive(Debug)]
pub struct Settings {
    pub consul: Consul,
    pub playlist: Playlist,
    pub http: Http,
}

impl Settings {
    pub fn from_file(file_path: &str) -> Result<Self, SettingsError> {
        let reader = File::open(file_path).map_err(|e| SettingsError::FileParse {
            path: Some(file_path.to_string()),
            cause: Box::new(e),
        })?;

        Settings::from_reader(reader)
    }

    pub fn from_reader<T: Read>(reader: T) -> Result<Self, SettingsError> {
        let file_settings: PartialSettings =
            serde_yaml::from_reader(reader).map_err(|e| SettingsError::FileParse {
                path: None,
                cause: Box::new(e),
            })?;

        Settings::merge(vec![file_settings, Default::default()])
    }

    pub fn merge(mut sources: Vec<PartialSettings>) -> Result<Self, SettingsError> {
        let consul_sources = sources
            .iter_mut()
            .map(|s| s.consul.take())
            .filter(|s| s.is_some())
            .map(|s| s.unwrap())
            .collect();

        let playlist_sources = sources
            .iter_mut()
            .map(|s| s.playlist.take())
            .filter(|s| s.is_some())
            .map(|s| s.unwrap())
            .collect();

        let http_sources = sources
            .iter_mut()
            .map(|s| s.http.take())
            .filter(|s| s.is_some())
            .map(|s| s.unwrap())
            .collect();

        Ok(Settings {
            consul: Consul::new(consul_sources)?,
            playlist: Playlist::new(playlist_sources)?,
            http: Http::new(http_sources)?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialSettings {
    consul: Option<PartialConsul>,
    playlist: Option<PartialPlaylist>,
    http: Option<PartialHttp>,
}

impl Default for PartialSettings {
    fn default() -> Self {
        PartialSettings {
            consul: Some(PartialConsul {
                base_url: None,
                update_interval: Some(Duration::from_secs(DEFAULT_CONSUL_UPDATE_INTERVAL_SECS)),
            }),
            playlist: Some(PartialPlaylist {
                upstream_base_url: None,
                segment_signing: Some(PartialSegmentSigning {
                    key: None,
                    duration: Some(Duration::from_secs(
                        DEFAULT_PLAYLIST_SEGMENT_SIGNING_DURATION_SECS,
                    )),
                }),
            }),
            http: Some(PartialHttp {
                socket: Some(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 2350)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_supply_only_required() {
        let yml = r#"
consul:
    base_url: "https://consul"
playlist:
    upstream_base_url: "https://playlist-upstream"
    segment_signing:
        key: "dis is key"
"#;

        println!("{}", yml);

        let settings = Settings::from_reader(yml.as_bytes()).unwrap();

        assert_eq!(settings.consul.base_url.as_str(), "https://consul/");
        assert_eq!(
            settings.consul.update_interval.as_secs(),
            DEFAULT_CONSUL_UPDATE_INTERVAL_SECS
        );
        assert_eq!(
            settings.playlist.upstream_base_url.as_str(),
            "https://playlist-upstream/"
        );
        assert_eq!(settings.playlist.segment_signing.key, "dis is key");
        assert_eq!(
            settings.playlist.segment_signing.duration.as_secs(),
            DEFAULT_PLAYLIST_SEGMENT_SIGNING_DURATION_SECS
        );
        assert_eq!(
            settings.http.socket,
            SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 2350)
        );
    }

    #[test]
    fn test_override_defaults() {
        let yml = r#"
consul:
    base_url: https://consul
    update_interval: 60m
playlist:
    upstream_base_url: https://playlist-upstream
    segment_signing:
        key: "dis is key"
        duration: 30m
http:
    socket: 8.8.8.8:33
"#;

        println!("{}", yml);

        let settings = Settings::from_reader(yml.as_bytes()).unwrap();

        assert_eq!(settings.consul.base_url.as_str(), "https://consul/");
        assert_eq!(settings.consul.update_interval.as_secs(), 60 * 60);
        assert_eq!(
            settings.playlist.upstream_base_url.as_str(),
            "https://playlist-upstream/"
        );
        assert_eq!(settings.playlist.segment_signing.key, "dis is key");
        assert_eq!(
            settings.playlist.segment_signing.duration.as_secs(),
            30 * 60
        );
        assert_eq!(
            settings.http.socket,
            SocketAddr::new(IpAddr::from_str("8.8.8.8").unwrap(), 33)
        );
    }
}
