use crate::edge_node_discovery::{EdgeNodeList, EdgeNodeProvider};
use crate::http::auth::Claims;
use crate::playlist::PlaylistRewriter;
use hls_m3u8::{MediaPlaylist, MediaSegment};
use log::warn;
use rand::seq::SliceRandom;
use rand::Rng;
use std::fmt;
use url::{ParseError, Url};

#[cfg(test)]
mod tests {
    use super::*;
    use hls_m3u8::MediaPlaylist;
    use std::borrow::Cow;
    use std::sync::Arc;
    use std::time::Duration;
    use url::Url;

    use rand::rngs::mock::StepRng;
    use rand::thread_rng;
    use rand::Rng;

    use crate::edge_node_discovery::{EdgeNode, EdgeNodeList};
    use crate::test_util::build_segment;

    struct MockEdgeNodeProvider {
        edge_nodes: Vec<Url>,
    }

    impl EdgeNodeProvider for MockEdgeNodeProvider {
        fn get_edge_nodes(&self, node_group: &str) -> EdgeNodeList {
            Arc::new(
                self.edge_nodes
                    .iter()
                    .map(|v| EdgeNode {
                        url: v.clone(),
                        group: String::from("group"),
                    })
                    .collect(),
            )
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

        let edge_nodes = vec![
            Url::parse("https://alpha.com:2323").unwrap(),
            Url::parse("https://beta.com").unwrap(),
            Url::parse("https://gamma.com").unwrap(),
        ];

        let rng_seed = thread_rng().gen();
        let mut rng = StepRng::new(rng_seed, rng_seed);
        let distri_rng = StepRng::new(rng_seed, rng_seed);

        // setup distributor
        let distributor = SegmentLoadDistributor::new(
            MockEdgeNodeProvider {
                edge_nodes: edge_nodes.clone(),
            },
            Box::new(move || distri_rng.clone()),
        );

        // rewrite
        let media_playlist = distributor.rewrite_playlist(media_playlist, "test");
        let uris: Vec<Cow<str>> = media_playlist
            .segments
            .values()
            .map(|seg| seg.uri().clone())
            .collect();

        // assert
        let file_endings = vec!["/23.ts", "/24.ts", "/25.ts"];

        let expected: Vec<String> = file_endings
            .iter()
            .map(|v| {
                edge_nodes
                    .choose(&mut rng)
                    .unwrap()
                    .join(v)
                    .unwrap()
                    .to_string()
            })
            .collect();

        assert_eq!(expected, uris)
    }
}

type RngProvider<U> = Box<dyn Fn() -> U + Send + Sync>;

pub struct SegmentLoadDistributor<T, U>
where
    T: EdgeNodeProvider,
    U: Rng + Send + Sync,
{
    edge_node_provider: T,
    rng_provider: RngProvider<U>,
}

impl<T, U> SegmentLoadDistributor<T, U>
where
    T: EdgeNodeProvider,
    U: Rng + Send + Sync,
{
    pub fn new(
        edge_node_provider: T,
        rng_provider: RngProvider<U>,
    ) -> SegmentLoadDistributor<T, U> {
        SegmentLoadDistributor {
            edge_node_provider,
            rng_provider,
        }
    }
}

impl<T, U> PlaylistRewriter for SegmentLoadDistributor<T, U>
where
    T: EdgeNodeProvider,
    U: Rng + Send + Sync,
{
    fn rewrite_playlist<'a>(
        &self,
        mut playlist: MediaPlaylist<'a>,
        node_group: &str,
    ) -> MediaPlaylist<'a> {
        let edge_nodes = self.edge_node_provider.get_edge_nodes(node_group);
        let rnd_edge_node_iter = RndEdgeNodeUrlIter::new(&edge_nodes, (self.rng_provider)());

        let edge_node_seg_iter = rnd_edge_node_iter.zip(playlist.segments.values_mut());

        for (edge_node, seg) in edge_node_seg_iter {
            match try_to_change_segment_uri_host(&seg, &edge_node) {
                Ok(uri) => {
                    seg.set_uri(uri);
                }
                Err(e) => warn!("Failed to change segment uri host: {}", e),
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
            HostChangeErr::Host(host) => write!(
                f,
                "Failed to set host to `{}`",
                host.as_ref().unwrap_or(&"None".to_string())
            ),
            HostChangeErr::Port(port) => {
                write!(f, "Failed to set port to `{}`", port.unwrap_or_default())
            }
            HostChangeErr::UrlParse(e) => e.fmt(f),
        }
    }
}

impl From<ParseError> for HostChangeErr {
    fn from(e: ParseError) -> Self {
        HostChangeErr::UrlParse(e)
    }
}

fn try_to_change_segment_uri_host(
    seg: &MediaSegment,
    edge_node: &Url,
) -> Result<String, HostChangeErr> {
    let mut seg_uri = edge_node.join(seg.uri())?;

    seg_uri
        .set_scheme(edge_node.scheme())
        .map_err(|_| HostChangeErr::Scheme(edge_node.scheme().to_string()))?;

    seg_uri
        .set_host(edge_node.host_str())
        .map_err(|_| HostChangeErr::Host(edge_node.host_str().map(|s| s.to_string())))?;

    seg_uri
        .set_port(edge_node.port())
        .map_err(|_| HostChangeErr::Port(edge_node.port()))?;

    Ok(seg_uri.into_string())
}

struct RndEdgeNodeUrlIter<'a, T: Rng> {
    edge_nodes: &'a EdgeNodeList,
    rng: T,
}

impl<'a, T: Rng> RndEdgeNodeUrlIter<'a, T> {
    fn new(edge_nodes: &'a EdgeNodeList, rng: T) -> Self {
        RndEdgeNodeUrlIter { edge_nodes, rng }
    }
}

impl<'a, T> Iterator for RndEdgeNodeUrlIter<'a, T>
where
    T: Rng,
{
    type Item = &'a Url;

    fn next(&mut self) -> Option<Self::Item> {
        self.edge_nodes.choose(&mut self.rng).map(|v| &v.url)
    }
}
