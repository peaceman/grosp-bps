use hls_m3u8::{MediaPlaylist};

mod segment_load_distributor;
mod segment_url_signer;

trait PlaylistRewriter {
    fn rewrite_playlist<'a>(&self, playlist: MediaPlaylist<'a>) -> MediaPlaylist<'a>;
}
