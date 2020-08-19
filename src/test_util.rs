use hls_m3u8::MediaSegment;
use std::time::Duration;

pub fn build_segment(uri: &'static str) -> MediaSegment {
    MediaSegment::builder()
        .duration(Duration::from_secs(3))
        .uri(uri)
        .build()
        .unwrap()
}
