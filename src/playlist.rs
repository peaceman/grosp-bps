#[cfg(test)]
mod tests {
    use super::*;
    use hls_m3u8::{MediaPlaylist, MediaSegment};
    use std::time::Duration;
    use std::borrow::Cow;

    fn build_segment(uri: &'static str) -> MediaSegment {
        MediaSegment::builder()
            .duration(Duration::from_secs(3))
            .uri(uri)
            .build()
            .unwrap()
    }

    struct MockEdgeNodeProvider {
        edge_nodes: Vec<String>,
    }

    impl EdgeNodeProvider for MockEdgeNodeProvider {
        fn get_edge_nodes(&self, amount: usize) -> Vec<String> {
            self.edge_nodes.clone()
        }
    }

    #[test]
    fn test_distribution() {
        // setup test playlist
        let media_playlist = MediaPlaylist::builder()
            .target_duration(Duration::from_secs(3))
            .push_segment(build_segment("http://example.com/23.ts"))
            .push_segment(build_segment("http://example.com/24.ts"))
            .push_segment(build_segment("http://example.com/25.ts"))
            .build()
            .unwrap();

        let distributor = SegmentLoadDistributor::new(
            MockEdgeNodeProvider {
                edge_nodes: vec![
                    String::from("https://alpha.com:2323"),
                    String::from("https://beta.com"),
                    String::from("htts://gammam"),
                ]
            }
        );

        let media_playlist = distributor.rewrite_playlist(media_playlist);
        let uris: Vec<Cow<str>> = media_playlist.segments.values()
            .map(|seg| seg.uri().clone())
            .collect();

        assert_eq!(
            vec![
                String::from("https://alpha.com:2323/23.ts"),
                String::from("https://beta.com/24.ts"),
                String::from("http://example.com/25.ts"),
            ],
            uris
        )
    }
}

use hls_m3u8::{MediaPlaylist, MediaSegment};
use url::{Url, ParseError};
use crate::edge_node_discovery::EdgeNodeProvider;

trait PlaylistRewriter {
    fn rewrite_playlist(&self, playlist: MediaPlaylist) -> MediaPlaylist;
}

struct SegmentLoadDistributor<T>
    where T: EdgeNodeProvider
{
    edge_node_provider: T,
}

impl <T> SegmentLoadDistributor<T>
    where T: EdgeNodeProvider
{
    fn new(edge_node_provider: T) -> SegmentLoadDistributor<T> {
        SegmentLoadDistributor {
            edge_node_provider
        }
    }
}

impl <T> SegmentLoadDistributor<T>
    where T: EdgeNodeProvider
{
    fn rewrite_playlist<'a>(&self, mut playlist: MediaPlaylist<'a>) -> MediaPlaylist<'a> {
        let edge_nodes = self.edge_node_provider
            .get_edge_nodes(playlist.segments.num_elements());

        let edge_node_seg_iter = edge_nodes.into_iter()
            .zip(playlist.segments.values_mut());

        for (edge_node, seg) in edge_node_seg_iter {
            if let Some(uri) = try_to_change_segment_uri_host(&seg, &edge_node) {
                seg.set_uri(uri);
            }
        }

        playlist
    }
}

fn try_to_change_segment_uri_host(seg: &MediaSegment, edge_node: &String) -> Option<String> {
    let edge_node_uri = Url::parse(&edge_node).ok()?;
    let mut seg_uri = Url::parse(seg.uri()).ok()?;

    seg_uri.set_scheme(edge_node_uri.scheme()).ok()?;
    seg_uri.set_host(edge_node_uri.host_str()).ok()?;
    seg_uri.set_port(edge_node_uri.port()).ok()?;

    Some(seg_uri.into_string())
}
