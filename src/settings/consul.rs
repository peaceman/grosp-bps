use serde::Deserialize;
use std::time::Duration;
use url::Url;

use super::SettingsError;

#[derive(Debug)]
pub struct Consul {
    pub base_url: Url,
    pub update_interval: Duration,
}

impl Consul {
    pub fn new(sources: Vec<PartialConsul>) -> Result<Self, SettingsError> {
        let merged: PartialConsul =
            sources
                .into_iter()
                .fold(Default::default(), |acc, x| PartialConsul {
                    base_url: acc.base_url.or(x.base_url),
                    update_interval: acc.update_interval.or(x.update_interval),
                });

        Ok(Consul {
            base_url: merged
                .base_url
                .ok_or_else(|| SettingsError::MissingValue("consul.base_url".to_string()))?,
            update_interval: merged.update_interval.ok_or_else(|| {
                SettingsError::MissingValue("consule.update_interval".to_string())
            })?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialConsul {
    pub base_url: Option<Url>,

    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub update_interval: Option<Duration>,
}

impl Default for PartialConsul {
    fn default() -> Self {
        PartialConsul {
            base_url: None,
            update_interval: None,
        }
    }
}
