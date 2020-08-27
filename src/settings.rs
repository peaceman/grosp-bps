use std::net::{SocketAddr, Ipv6Addr, IpAddr};
use url::Url;
use std::time::Duration;
use std::result::Result;
use std::error::Error;
use std::fs::File;
use std::fmt;
use std::io::Read;

const DEFAULT_CONSUL_UPDATE_INTERVAL_SECS: u64 = 1;
const DEFAULT_PLAYLIST_SEGMENT_SIGNING_DURATION_SECS: u64 = 60;

mod partial {
    use std::time::Duration;
    use std::net::{Ipv6Addr, IpAddr, SocketAddr};
    use url::Url;
    use serde::{Deserialize};
    use super::DEFAULT_CONSUL_UPDATE_INTERVAL_SECS;
    use super::DEFAULT_PLAYLIST_SEGMENT_SIGNING_DURATION_SECS;

    #[derive(Debug, Deserialize)]
    pub struct Settings {
        pub consul: Option<Consul>,
        pub playlist: Option<Playlist>,
        pub http: Option<Http>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Consul {
        pub base_url: Option<Url>,

        #[serde(default)]
        #[serde(with = "humantime_serde")]
        pub update_interval: Option<Duration>,
    }

    impl Default for Consul {
        fn default() -> Self {
            Consul {
                base_url: None,
                update_interval: None,
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct Playlist {
        pub upstream_base_url: Option<Url>,
        pub segment_signing: Option<SegmentSigning>,
    }

    impl Default for Playlist {
        fn default() -> Self {
            Playlist {
                upstream_base_url: None,
                segment_signing: None,
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct SegmentSigning {
        pub key: Option<String>,

        #[serde(default)]
        #[serde(with = "humantime_serde")]
        pub duration: Option<Duration>,
    }

    impl Default for SegmentSigning {
        fn default() -> Self {
            SegmentSigning {
                key: None,
                duration: None,
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct Http {
        pub socket: Option<SocketAddr>,
    }

    impl Default for Http {
        fn default() -> Self {
            Http {
                socket: None
            }
        }
    }

    impl Default for Settings {
        fn default() -> Self {
            Settings {
                consul: Some(Consul {
                    base_url: None,
                    update_interval: Some(Duration::from_secs(DEFAULT_CONSUL_UPDATE_INTERVAL_SECS))
                }),
                playlist: Some(Playlist {
                    upstream_base_url: None,
                    segment_signing: Some(SegmentSigning {
                        key: None,
                        duration:  Some(Duration::from_secs(DEFAULT_PLAYLIST_SEGMENT_SIGNING_DURATION_SECS))
                    }),
                }),
                http: Some(Http {
                    socket: Some(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 2350))
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct Settings {
    consul: Consul,
    playlist: Playlist,
    http: Http,
}

#[derive(Debug)]
pub struct Consul {
    base_url: Url,
    update_interval: Duration,
}

#[derive(Debug)]
pub struct Playlist {
    upstream_base_url: Url,
    segment_signing: SegmentSigning,
}

#[derive(Debug)]
pub struct SegmentSigning {
    key: String,
    duration: Duration,
}

#[derive(Debug)]
pub struct Http {
    socket: SocketAddr,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            socket: SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 2350),
        }
    }
}

pub enum SettingsError {
    FileParse {
        path: Option<String>,
        cause: Box<dyn Error + Send + Sync>,
    },
    Message(String),
    MissingValue(String),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::FileParse { path, cause } => {
                write!(f, "FileParse {}", cause)?;

                if let Some(path) = path {
                    write!(f, " in {}", path)?;
                }

                Ok(())
            },
            SettingsError::Message(msg) => write!(f, "{}", msg),
            SettingsError::MissingValue(path) => write!(f, "Missing settings value at {}", path),
        }
    }
}

impl std::fmt::Debug for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Consul {
    fn new(sources: Vec<partial::Consul>) -> Result<Self, SettingsError> {
        let merged: partial::Consul = sources.into_iter().fold(Default::default(), |acc, x| {
            partial::Consul {
                base_url: acc.base_url.or(x.base_url),
                update_interval: acc.update_interval.or(x.update_interval)
            }
        });

        Ok(Consul {
            base_url: merged.base_url.ok_or_else(|| SettingsError::MissingValue("consul.base_url".to_string()))?,
            update_interval: merged.update_interval.ok_or_else(|| SettingsError::MissingValue("consule.update_interval".to_string()))?,
        })
    }
}

impl Playlist {
    fn new(mut sources: Vec<partial::Playlist>) -> Result<Self, SettingsError> {
        let merged: partial::Playlist = sources.iter_mut().fold(Default::default(), |acc, x| {
            partial::Playlist {
                upstream_base_url: acc.upstream_base_url.or_else(|| x.upstream_base_url.take()),
                segment_signing: None,
            }
        });

        let ss = SegmentSigning::new(sources.iter_mut()
            .map(|s| s.segment_signing.take())
            .filter(|s| s.is_some())
            .map(|s| s.unwrap())
            .collect())?;

        Ok(Playlist {
            upstream_base_url: merged.upstream_base_url.ok_or_else(|| SettingsError::MissingValue("playlist.upstream_base_url".to_string()))?,
            segment_signing: ss,
        })
    }
}

impl SegmentSigning {
    fn new(sources: Vec<partial::SegmentSigning>) -> Result<Self, SettingsError> {
        let merged: partial::SegmentSigning = sources.into_iter().fold(Default::default(), |acc, x| {
            partial::SegmentSigning {
                key: acc.key.or(x.key),
                duration: acc.duration.or(x.duration),
            }
        });

        Ok(SegmentSigning {
            key: merged.key.ok_or_else(|| SettingsError::MissingValue("playlist.segment_signing_key".to_string()))?,
            duration: merged.duration.ok_or_else(|| SettingsError::MissingValue("playlist.segment_signing.duration".to_string()))?,
        })
    }
}

impl Http {
    fn new(sources: Vec<partial::Http>) -> Result<Self, SettingsError> {
        let merged: partial::Http = sources.iter().fold(Default::default(), |acc, x| {
            partial::Http {
                socket: acc.socket.or(x.socket),
            }
        });

        Ok(Http {
            socket: merged.socket.ok_or_else(|| SettingsError::MissingValue("http.socket".to_string()))?,
        })
    }
}

impl Settings {
    pub fn from_file(file_path: &str) -> Result<Self, SettingsError> {
        let reader = File::open(file_path)
            .map_err(|e| SettingsError::FileParse {
                path: Some(file_path.to_string()),
                cause: Box::new(e),
            })?;

        Settings::from_reader(reader)
    }

    pub fn from_reader<T: Read>(reader: T) -> Result<Self, SettingsError> {
        let file_settings: partial::Settings = serde_yaml::from_reader(reader)
            .map_err(|e| SettingsError::FileParse {
                path: None,
                cause: Box::new(e),
            })?;

        Settings::merge(vec![file_settings, Default::default()])
    }

    pub fn merge(mut sources: Vec<partial::Settings>) -> Result<Self, SettingsError> {
        let consul_sources = sources.iter_mut()
            .map(|s| s.consul.take())
            .filter(|s| s.is_some())
            .map(|s| s.unwrap())
            .collect();

        let playlist_sources = sources.iter_mut()
            .map(|s| s.playlist.take())
            .filter(|s| s.is_some())
            .map(|s| s.unwrap())
            .collect();

        let http_sources = sources.iter_mut()
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

        let settings = Settings::from_reader(yml.as_bytes())
            .unwrap();

        assert_eq!(settings.consul.base_url.as_str(), "https://consul/");
        assert_eq!(settings.consul.update_interval.as_secs(), DEFAULT_CONSUL_UPDATE_INTERVAL_SECS);
        assert_eq!(settings.playlist.upstream_base_url.as_str(), "https://playlist-upstream/");
        assert_eq!(settings.playlist.segment_signing.key, "dis is key");
        assert_eq!(settings.playlist.segment_signing.duration.as_secs(), DEFAULT_PLAYLIST_SEGMENT_SIGNING_DURATION_SECS);
        assert_eq!(settings.http.socket, SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 2350));
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

        let settings = Settings::from_reader(yml.as_bytes())
            .unwrap();

        assert_eq!(settings.consul.base_url.as_str(), "https://consul/");
        assert_eq!(settings.consul.update_interval.as_secs(), 60 * 60);
        assert_eq!(settings.playlist.upstream_base_url.as_str(), "https://playlist-upstream/");
        assert_eq!(settings.playlist.segment_signing.key, "dis is key");
        assert_eq!(settings.playlist.segment_signing.duration.as_secs(), 30 * 60);
        assert_eq!(settings.http.socket, SocketAddr::new(IpAddr::from_str("8.8.8.8").unwrap(), 33));
    }
}
