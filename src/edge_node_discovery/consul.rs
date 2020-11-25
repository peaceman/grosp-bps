use consul_api_client::Client;
use log::{error, info, warn};
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;
use tokio::time;
use url::Url;

use super::{EdgeNode, EdgeNodeList, EdgeNodeProvider};
use consul_api_client::health::{Health, ServiceEntry};
use std::convert::{TryFrom, TryInto};

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
    pub fn new(consul: Client, update_interval: Duration) -> Self {
        let edge_nodes = Arc::new(RwLock::new(Arc::new(vec![])));

        let provider = ConsulEdgeNodeProvider {
            edge_nodes: Arc::clone(&edge_nodes),
        };

        start_update_edge_nodes_loop(Arc::downgrade(&edge_nodes), consul, update_interval);

        provider
    }

    fn current_edge_nodes(&self) -> EdgeNodeList {
        Arc::clone(&self.edge_nodes.read().unwrap())
    }
}

fn start_update_edge_nodes_loop(
    edge_nodes: Weak<EdgeNodeStorage>,
    consul: Client,
    update_interval: Duration,
) {
    info!("Start update edge nodes loop");

    tokio::spawn(async move { update_edge_nodes_loop(edge_nodes, consul, update_interval).await });
}

async fn update_edge_nodes_loop(
    edge_nodes: Weak<EdgeNodeStorage>,
    client: Client,
    update_interval: Duration,
) {
    let mut interval = time::interval(update_interval);

    loop {
        interval.tick().await;

        let edge_nodes = match edge_nodes.upgrade() {
            Some(provider) => provider,
            None => {
                info!("Couldn't get reference to the edge node storage, ending update loop");
                break;
            }
        };

        match fetch_edge_nodes_from_consul(&client).await {
            Ok(new_edge_nodes) => {
                info!("Updating edge nodes from consul: {:?}", &new_edge_nodes);
                *edge_nodes.write().unwrap() = Arc::new(new_edge_nodes)
            }
            Err(e) => {
                error!("Failed to update edge nodes from consul: {}", e);
                continue;
            }
        }
    }
}

async fn fetch_edge_nodes_from_consul(client: &Client) -> anyhow::Result<Vec<EdgeNode>> {
    let service_name = "edge"; // todo configuration
    let (services, _meta) = client
        .service(service_name, Some("state=active"), true, None, None)
        .await
        .map_err(anyhow::Error::new)?;

    let edge_nodes = services
        .into_iter()
        .filter_map(try_to_convert_service_entry_to_edge_node)
        .collect();

    Ok(edge_nodes)
}

fn try_to_convert_service_entry_to_edge_node(service_entry: ServiceEntry) -> Option<EdgeNode> {
    let se = &service_entry;

    let en: Result<EdgeNode, _> = se.try_into();

    en.map(Some).unwrap_or_else(|e| {
        warn!(
            "Failed to convert consul service entry into edge node; Error: {} for node: {}",
            e, service_entry.Node.Node
        );
        None
    })
}

impl TryFrom<&ServiceEntry> for EdgeNode {
    type Error = &'static str;

    fn try_from(value: &ServiceEntry) -> Result<Self, Self::Error> {
        let url = value
            .Service
            .Meta
            .get("edge_url")
            .ok_or("Missing edge_url in service meta")
            .and_then(|url| Url::parse(url.as_ref()).map_err(|_| "Failed to parse edge_url"))?;

        let group = value
            .Service
            .Meta
            .get("node_group")
            .ok_or("Missing node_group in service meta")?;

        Ok(Self {
            url,
            group: group.clone(),
        })
    }
}

#[cfg(test)]
mod tests {}
