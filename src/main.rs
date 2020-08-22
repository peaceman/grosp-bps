use bps::http::create_routes;
use reqwest::{Client, Url};
use bps::playlist::{SegmentUrlSigner, HmacUrlSigner, SegmentLoadDistributor, CombinedPlaylistRewriter, PlaylistRewriter};
use bps::edge_node_discovery::ConsulEdgeNodeProvider;
use bps::http::HttpClient;
use std::time::Duration;
use std::sync::Arc;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[tokio::main]
async fn main() {
    let updating_edge_nodes_provider = ConsulEdgeNodeProvider::new(HttpClient::new(
        reqwest::Client::new(),
        Url::parse("http://localhost:8500").unwrap(),
    ));
    let http_client = Client::new();
    let base_url = Url::parse("https://live.vizzywig.de/live/").unwrap();

    let segment_signer = SegmentUrlSigner::new(
        HmacUrlSigner::new("foobar".to_string()),
        Duration::from_secs(300),
    );

    // let edge_node_provider = vec![
    //     "http://relay.hsp-events.de:7002".to_string(),
    // ];

    let segment_load_distributor = SegmentLoadDistributor::new(updating_edge_nodes_provider, StdRng::from_entropy());

    let rewriters: Vec<Box<dyn PlaylistRewriter>> = vec![
        Box::new(segment_load_distributor),
        Box::new(segment_signer),
    ];

    let routes = create_routes(http_client, base_url, Arc::new(CombinedPlaylistRewriter::new(
        rewriters
    )));

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3000))
        .await;
}
