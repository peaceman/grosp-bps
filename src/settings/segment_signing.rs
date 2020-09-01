use serde::Deserialize;
use std::time::Duration;

use super::SettingsError;

#[derive(Debug)]
pub struct SegmentSigning {
    pub key: String,
    pub duration: Duration,
}

impl SegmentSigning {
    pub fn new(sources: Vec<PartialSegmentSigning>) -> Result<Self, SettingsError> {
        let merged: PartialSegmentSigning =
            sources
                .into_iter()
                .fold(Default::default(), |acc, x| PartialSegmentSigning {
                    key: acc.key.or(x.key),
                    duration: acc.duration.or(x.duration),
                });

        Ok(SegmentSigning {
            key: merged.key.ok_or_else(|| {
                SettingsError::MissingValue("playlist.segment_signing_key".to_string())
            })?,
            duration: merged.duration.ok_or_else(|| {
                SettingsError::MissingValue("playlist.segment_signing.duration".to_string())
            })?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialSegmentSigning {
    pub key: Option<String>,

    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub duration: Option<Duration>,
}

impl Default for PartialSegmentSigning {
    fn default() -> Self {
        PartialSegmentSigning {
            key: None,
            duration: None,
        }
    }
}
