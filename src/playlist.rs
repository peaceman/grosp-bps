mod segment_load_distributor;
mod segment_url_signer;

use hls_m3u8::MediaPlaylist;

pub use segment_load_distributor::SegmentLoadDistributor;
pub use segment_url_signer::HmacUrlSigner;
pub use segment_url_signer::SegmentUrlSigner;

pub trait PlaylistRewriter: Send + Sync {
    fn rewrite_playlist<'a>(&self, playlist: MediaPlaylist<'a>) -> MediaPlaylist<'a>;
}

pub struct CombinedPlaylistRewriter {
    rewriters: Vec<Box<dyn PlaylistRewriter>>,
}

impl CombinedPlaylistRewriter {
    pub fn new(rewriters: Vec<Box<dyn PlaylistRewriter>>) -> Self {
        CombinedPlaylistRewriter { rewriters }
    }
}

impl PlaylistRewriter for CombinedPlaylistRewriter {
    fn rewrite_playlist<'a>(&self, playlist: MediaPlaylist<'a>) -> MediaPlaylist<'a> {
        let mut playlist = playlist;

        for rewriter in self.rewriters.iter() {
            playlist = rewriter.rewrite_playlist(playlist);
        }

        playlist
    }
}

#[cfg(test)]
mod tests {
    use hls_m3u8::MediaSegment;
    use std::time::Duration;

    #[allow(dead_code)]
    pub fn build_segment(uri: &'static str) -> MediaSegment {
        MediaSegment::builder()
            .duration(Duration::from_secs(3))
            .uri(uri)
            .build()
            .unwrap()
    }
}
