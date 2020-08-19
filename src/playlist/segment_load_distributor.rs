use hls_m3u8::{MediaPlaylist, MediaSegment};
use crate::edge_node_discovery::EdgeNodeProvider;
use url::{Url, ParseError};
use crate::playlist::PlaylistRewriter;
use std::fmt;

#[cfg(test)]
mod tests {
    use super::*;
    use hls_m3u8::{MediaPlaylist, MediaSegment};
    use std::time::Duration;
    use std::borrow::Cow;
    use crate::test_util::build_segment;

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
            .push_segment(build_segment("/25.ts"))
            .build()
            .unwrap();

        // setup distributor
        let distributor = SegmentLoadDistributor::new(
            MockEdgeNodeProvider {
                edge_nodes: vec![
                    String::from("https://alpha.com:2323"),
                    String::from("https://beta.com"),
                    String::from("https://gamma.com"),
                ]
            }
        );

        // rewrite
        let media_playlist = distributor.rewrite_playlist(media_playlist);
        let uris: Vec<Cow<str>> = media_playlist.segments.values()
            .map(|seg| seg.uri().clone())
            .collect();

        // assert
        assert_eq!(
            vec![
                String::from("https://alpha.com:2323/23.ts"),
                String::from("https://beta.com/24.ts"),
                String::from("https://gamma.com/25.ts"),
            ],
            uris
        )
    }

    #[test]
    fn test_distribution_with_url_errors() {
        // setup test playlist
        let media_playlist = MediaPlaylist::builder()
            .target_duration(Duration::from_secs(3))
            .push_segment(build_segment("http://example.com/23.ts"))
            .push_segment(build_segment("http://example.com/24.ts"))
            .build()
            .unwrap();

        // setup distributor
        let distributor = SegmentLoadDistributor::new(
            MockEdgeNodeProvider {
                edge_nodes: vec![
                    String::from("not sure"),
                    String::from("https://beta.com"),
                    String::from("htts:/d/gammam"),
                ]
            }
        );

        // rewrite
        let media_playlist = distributor.rewrite_playlist(media_playlist);
        let uris: Vec<Cow<str>> = media_playlist.segments.values()
            .map(|seg| seg.uri().clone())
            .collect();

        // assert
        assert_eq!(
            vec![
                String::from("http://example.com/23.ts"),
                String::from("https://beta.com/24.ts"),
            ],
            uris
        )
    }
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

impl <T> PlaylistRewriter for SegmentLoadDistributor<T>
    where T: EdgeNodeProvider
{
    fn rewrite_playlist<'a>(&self, mut playlist: MediaPlaylist<'a>) -> MediaPlaylist<'a> {
        let edge_nodes = self.edge_node_provider
            .get_edge_nodes(playlist.segments.num_elements());

        let edge_node_seg_iter = edge_nodes.into_iter()
            .zip(playlist.segments.values_mut());

        for (edge_node, seg) in edge_node_seg_iter {
            match try_to_change_segment_uri_host(&seg, &edge_node) {
                Ok(uri) => {
                    seg.set_uri(uri);
                },
                Err(e) => eprintln!("Failed to change segment uri host: {}", e),
            }
        }

        playlist
    }
}

#[derive(Debug, Clone)]
enum HostChangeErr {
    Scheme(String),
    Host(Option<String>),
    Port(Option<u16>),
    UrlParse(ParseError),
}

impl fmt::Display for HostChangeErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HostChangeErr::Scheme(scheme) => write!(f, "Failed to set scheme to `{}`", scheme),
            HostChangeErr::Host(host) => write!(f, "Failed to set host to `{}`", host.as_ref().unwrap_or(&"None".to_string())),
            HostChangeErr::Port(port) => write!(f, "Failed to set port to `{}`", port.unwrap_or_default()),
            HostChangeErr::UrlParse(e) => e.fmt(f),
        }
    }
}

impl From<ParseError> for HostChangeErr {
    fn from(e: ParseError) -> Self {
        HostChangeErr::UrlParse(e)
    }
}

fn try_to_change_segment_uri_host(seg: &MediaSegment, edge_node: &String) -> Result<String, HostChangeErr> {
    let edge_node_uri = Url::parse(&edge_node)?;
    let mut seg_uri = edge_node_uri.join(seg.uri())?;

    seg_uri.set_scheme(edge_node_uri.scheme())
        .map_err(|_| HostChangeErr::Scheme(edge_node_uri.scheme().to_string()))?;

    seg_uri.set_host(edge_node_uri.host_str())
        .map_err(|_| HostChangeErr::Host(edge_node_uri.host_str().map(|s| s.to_string())))?;

    seg_uri.set_port(edge_node_uri.port())
        .map_err(|_| HostChangeErr::Port(edge_node_uri.port()))?;

    Ok(seg_uri.into_string())
}
