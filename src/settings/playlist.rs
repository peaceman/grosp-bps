use serde::Deserialize;
use url::Url;

use super::segment_signing::SegmentSigning;
use super::segment_signing::PartialSegmentSigning;
use super::SettingsError;

#[derive(Debug)]
pub struct Playlist {
    pub upstream_base_url: Url,
    pub segment_signing: SegmentSigning,
}

impl Playlist {
    pub fn new(mut sources: Vec<PartialPlaylist>) -> Result<Self, SettingsError> {
        let merged: PartialPlaylist = sources.iter_mut().fold(Default::default(), |acc, x| {
            PartialPlaylist {
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

#[derive(Debug, Deserialize)]
pub struct PartialPlaylist {
    pub upstream_base_url: Option<Url>,
    pub segment_signing: Option<PartialSegmentSigning>,
}

impl Default for PartialPlaylist {
    fn default() -> Self {
        PartialPlaylist {
            upstream_base_url: None,
            segment_signing: None,
        }
    }
}
