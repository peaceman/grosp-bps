mod problem;

use anyhow::Context;
use hls_m3u8::MediaPlaylist;
use log::debug;
use reqwest::{Client, Url};
use std::sync::Arc;
use warp::{filters::BoxedFilter, http::Response, Filter, Rejection, Reply};

use self::problem::from_anyhow;
use crate::playlist::PlaylistRewriter;

pub fn create_routes(
    http_client: Client,
    base_url: Url,
    playlist_rewriter: Arc<dyn PlaylistRewriter>,
) -> BoxedFilter<(impl Reply,)> {
    let http_client = warp::any().map(move || http_client.clone());
    let base_url = warp::any().map(move || base_url.clone());
    let playlist_rewriter = warp::any().map(move || Arc::clone(&playlist_rewriter));

    let get_playlist = warp::path("playlist")
        .and(warp::get())
        .and(warp::path::tail())
        .and(http_client)
        .and(base_url)
        .and(playlist_rewriter)
        .and_then(get_playlist);

    let healthz = warp::path("healthz").map(|| "🧩");

    healthz.or(get_playlist).boxed()
}

#[derive(Debug)]
struct FetchError {
    msg: String,
}

impl warp::reject::Reject for FetchError {}

async fn get_playlist(
    tail: warp::path::Tail,
    http_client: Client,
    base_url: Url,
    playlist_rewriter: Arc<dyn PlaylistRewriter>,
) -> Result<Box<dyn Reply>, Rejection> {
    let upstream_playlist_url =
        build_playlist_url(&tail, &base_url).map_err(warp::reject::custom)?;

    debug!("upstream playlist url: {}", upstream_playlist_url);

    let upstream_response_body = fetch_playlist_from_upstream(&http_client, &upstream_playlist_url)
        .await
        .map_err(warp::reject::custom)?;

    let response = match upstream_response_body.parse::<MediaPlaylist>() {
        Err(_) => upstream_response_body,
        Ok(playlist) => playlist_rewriter.rewrite_playlist(playlist).to_string(),
    };

    Ok(Box::new(Response::new(response)))
}

fn build_playlist_url(
    tail: &warp::path::Tail,
    base_url: &Url,
) -> Result<Url, impl warp::reject::Reject> {
    base_url
        .join(tail.as_str())
        .with_context(|| {
            format!(
                "Failed to build upstream playlist url from base url `{}` and path tail `{}`",
                base_url.as_str(),
                tail.as_str()
            )
        })
        .map_err(|e| from_anyhow(e, 400))
}

async fn fetch_playlist_from_upstream(
    http_client: &Client,
    url: &Url,
) -> Result<String, impl warp::reject::Reject> {
    http_client
        .get(url.clone())
        .send()
        .await
        .with_context(|| {
            format!(
                "Failed to retrieve playlist from upstream url `{}`",
                url.as_str()
            )
        })
        .map_err(|e| from_anyhow(e, 400))?
        .text()
        .await
        .with_context(|| {
            format!(
                "Failed to retrieve body from upstream playlist response from url `{}`",
                url.as_str()
            )
        })
        .map_err(|e| from_anyhow(e, 400))
}
