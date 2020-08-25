use std::sync::{RwLock, Arc, Weak};
use tokio::time;
use serde_json::Value;
use url::Url;
use anyhow::{Context};
use log::{info, warn, error};

use crate::http::HttpClient;
use super::{EdgeNodeProvider, EdgeNode, EdgeNodeList};

type EdgeNodeStorage = RwLock<EdgeNodeList>;

pub struct ConsulEdgeNodeProvider {
    edge_nodes: Arc<EdgeNodeStorage>,
}

impl EdgeNodeProvider for ConsulEdgeNodeProvider {
    fn get_edge_nodes(&self) -> EdgeNodeList {
        self.current_edge_nodes()
    }
}

impl ConsulEdgeNodeProvider {
    pub fn new(http_client: HttpClient) -> Self {
        let edge_nodes = Arc::new(RwLock::new(Arc::new(vec![])));

        let provider = ConsulEdgeNodeProvider {
            edge_nodes: Arc::clone(&edge_nodes),
        };

        start_update_edge_nodes_loop(Arc::downgrade(&edge_nodes), http_client);

        provider
    }

    fn current_edge_nodes(&self) -> EdgeNodeList {
        Arc::clone(&self.edge_nodes.read().unwrap())
    }
}

fn start_update_edge_nodes_loop(edge_nodes: Weak<EdgeNodeStorage>, http_client: HttpClient) {
    info!("Start update edge nodes loop");

    tokio::spawn(async move {
        update_edge_nodes_loop(edge_nodes, http_client).await
    });
}

async fn update_edge_nodes_loop(edge_nodes: Weak<EdgeNodeStorage>, http_client: HttpClient) {
    let mut interval = time::interval(time::Duration::from_secs(1));

    loop {
        interval.tick().await;

        let edge_nodes = match edge_nodes.upgrade() {
            Some(provider) => provider,
            None => {
                info!("Couldn't get reference to the edge node storage, ending update loop");
                break
            },
        };

        match fetch_edge_nodes_from_consul(&http_client).await {
            Ok(new_edge_nodes) => {
                info!("Updating edge nodes from consul: {:?}", &new_edge_nodes);
                *edge_nodes.write().unwrap() = Arc::new(new_edge_nodes)
            },
            Err(e) => {
                error!("Failed to update edge nodes from consul: {}", e);
                continue
            },
        }
    }
}

async fn fetch_edge_nodes_from_consul(http_client: &HttpClient)
    -> anyhow::Result<Vec<EdgeNode>> {
        let response_text = http_client.get("v1/health/service/edge?passing")
            .send()
            .await
            .with_context(|| "Failed to retrieve services from consul")?
            .text()
            .await
            .with_context(|| "Failed to retrieve body from consul services response")?;

    Ok(parse_edge_nodes_from_consul_json(&response_text))
}

fn parse_edge_nodes_from_consul_json(json: &str) -> Vec<EdgeNode> {
    let root: Value = match serde_json::from_str(json) {
        Err(e) => {
            error!("Failed to parse json from consul {}", e);
            return vec![];
        },
        Ok(v) => v,
    };

    root.as_array()
        .map(|a| a.iter()
            .map(|info| {
                info.get("Service")
                    .and_then(|v| v.get("Meta"))
                    .and_then(|v| v.get("edge_url"))
                    .and_then(|v| v.as_str())
                    .and_then(|v| Url::parse(v)
                        .map(|v| Some(EdgeNode { url: v }))
                        .unwrap_or_else(|e| {
                            warn!("Failed to parse edge node url {} {}", v, e);
                            None
                        })
                    )
            })
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .collect()
        )
        .unwrap_or_else(Vec::new)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_nodes_from_consul_json_error() {
        let result = parse_edge_nodes_from_consul_json("{}");

        assert!(result.is_empty(), "the parse result is not empty");
    }

    #[test]
    fn test_edge_nodes_from_consul_json_success() {
        let result = parse_edge_nodes_from_consul_json(r#"[
            {
                "Service": {
                    "Meta": {
                        "edge_url": "https://example.com"
                    }
                }
            },
            {
                "Service": {
                    "Meta": {
                        "edge_url": "is dis url?"
                    }
                }
            }
        ]"#);

        assert_eq!(
            vec![EdgeNode { url: Url::parse("https://example.com").unwrap() }],
            result
        )
    }
}
