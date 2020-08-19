use hls_m3u8::{MediaPlaylist, MediaSegment};
use std::time::Duration;

mod segment_load_distributor;
mod segment_url_signer;

trait PlaylistRewriter {
    fn rewrite_playlist<'a>(&self, playlist: MediaPlaylist<'a>) -> MediaPlaylist<'a>;
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn build_segment(uri: &'static str) -> MediaSegment {
        MediaSegment::builder()
            .duration(Duration::from_secs(3))
            .uri(uri)
            .build()
            .unwrap()
    }
}
