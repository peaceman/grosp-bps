use crate::http::auth::Claims;
use crate::playlist::PlaylistRewriter;
use hls_m3u8::MediaPlaylist;
use hmac::{Hmac, Mac, NewMac};
use log::{error, warn};
use sha2::Sha256;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::build_segment;
    use std::borrow::Cow;

    struct MockUrlSigner;
    impl UrlSigner for MockUrlSigner {
        fn sign(&self, mut url: Url, _expiry_timestamp: u64) -> Url {
            url.query_pairs_mut().append_pair("foo", "bar");

            url
        }
    }

    #[test]
    fn test_hmac_url_signing() {
        let signer = HmacUrlSigner::new("foobar".to_string());
        let url = Url::parse("https://example.com/23.ts").unwrap();
        let url = signer.sign(url, 23);

        assert_eq!(
            "https://example.com/23.ts?e=23&h=e5030a591d2dd923f90d29600b0c02e458c0bc344b1ad8eb71a26cf636988b62",
            url.into_string()
        );
    }

    #[test]
    fn test_segment_url_signer() {
        // setup test playlist
        let media_playlist = MediaPlaylist::builder()
            .target_duration(Duration::from_secs(3))
            .push_segment(build_segment("http://example.com/23.ts"))
            .push_segment(build_segment("http://example.com/24.ts"))
            .build()
            .unwrap();

        // setup signer
        let signer = SegmentUrlSigner {
            signer: MockUrlSigner,
            expiry_duration: Duration::from_secs(3),
        };

        // rewrite
        let media_playlist = signer.rewrite_playlist(media_playlist);
        let uris: Vec<Cow<str>> = media_playlist
            .segments
            .values()
            .map(|seg| seg.uri().clone())
            .collect();

        // assert
        assert_eq!(
            vec![
                String::from("http://example.com/23.ts?foo=bar"),
                String::from("http://example.com/24.ts?foo=bar"),
            ],
            uris
        )
    }
}

pub trait UrlSigner: Send + Sync {
    fn sign(&self, url: Url, expiry_timestamp: u64) -> Url;
}

pub struct HmacUrlSigner {
    key: String,
}

impl HmacUrlSigner {
    pub fn new(key: String) -> HmacUrlSigner {
        HmacUrlSigner { key }
    }

    fn new_hmac(&self) -> Hmac<Sha256> {
        Hmac::<Sha256>::new_varkey(self.key.as_bytes()).expect("HMAC can take key of any size")
    }
}

impl UrlSigner for HmacUrlSigner {
    fn sign(&self, mut url: Url, expiry_timestamp: u64) -> Url {
        let mut hmac = self.new_hmac();

        let mut content_to_sign = String::from(url.path());
        content_to_sign.push_str(&expiry_timestamp.to_string());

        hmac.update(&content_to_sign.as_bytes());
        let signature = hmac.finalize();
        let signature = hex::encode(signature.into_bytes());

        url.query_pairs_mut()
            .append_pair("e", &expiry_timestamp.to_string())
            .append_pair("h", &signature);

        url
    }
}

pub struct SegmentUrlSigner<T>
where
    T: UrlSigner,
{
    signer: T,
    expiry_duration: Duration,
}

impl<T> SegmentUrlSigner<T>
where
    T: UrlSigner,
{
    pub fn new(signer: T, expiry_duration: Duration) -> SegmentUrlSigner<T> {
        SegmentUrlSigner {
            signer,
            expiry_duration,
        }
    }
}

impl<T> PlaylistRewriter for SegmentUrlSigner<T>
where
    T: UrlSigner,
{
    fn rewrite_playlist<'a>(
        &self,
        mut playlist: MediaPlaylist<'a>,
        claims: &Claims,
    ) -> MediaPlaylist<'a> {
        let valid_until = (SystemTime::now() + self.expiry_duration).duration_since(UNIX_EPOCH);

        // skip playlist modification if we cant get a valid expiry unix timestamp
        if valid_until.is_err() {
            error!(
                "Failed to get a valid expiry unix timestamp: {}",
                valid_until.unwrap_err()
            );
            return playlist;
        }

        let valid_until = valid_until.unwrap().as_secs();

        for seg in playlist.segments.values_mut() {
            match Url::parse(seg.uri()) {
                Ok(url) => {
                    let signed_url = self.signer.sign(url, valid_until);
                    seg.set_uri(signed_url.into_string());
                }
                Err(e) => warn!("Failed to parse URL: {} Err: {}", seg.uri(), e),
            }
        }

        playlist
    }
}
