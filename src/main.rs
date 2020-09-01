use balancing_playlist_spreader::{
    edge_node_discovery::ConsulEdgeNodeProvider,
    http::{create_routes, HttpClient},
    playlist::{
        CombinedPlaylistRewriter, HmacUrlSigner, PlaylistRewriter, SegmentLoadDistributor,
        SegmentUrlSigner,
    },
    settings::Settings,
};

use rand::rngs::StdRng;
use rand::SeedableRng;
use reqwest::Client;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    env_logger::init();

    let settings = Settings::from_file("config.yml").expect("Failed to load config");

    let updating_edge_nodes_provider = ConsulEdgeNodeProvider::new(
        HttpClient::new(Client::new(), settings.consul.base_url),
        settings.consul.update_interval,
    );

    let segment_signer = SegmentUrlSigner::new(
        HmacUrlSigner::new(settings.playlist.segment_signing.key),
        settings.playlist.segment_signing.duration,
    );

    let segment_load_distributor =
        SegmentLoadDistributor::new(updating_edge_nodes_provider, Box::new(StdRng::from_entropy));

    let rewriters: Vec<Box<dyn PlaylistRewriter>> =
        vec![Box::new(segment_load_distributor), Box::new(segment_signer)];

    let routes = create_routes(
        Client::new(),
        settings.playlist.upstream_base_url,
        Arc::new(CombinedPlaylistRewriter::new(rewriters)),
    );

    warp::serve(routes).run(settings.http.socket).await;
}
