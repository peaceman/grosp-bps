pub mod consul;

use std::sync::Arc;
use url::Url;

pub use consul::ConsulEdgeNodeProvider;

pub trait EdgeNodeProvider: Send + Sync {
    fn get_edge_nodes(&self) -> EdgeNodeList;
}

pub type EdgeNodeList = Arc<Vec<EdgeNode>>;

#[derive(Debug, PartialEq)]
pub struct EdgeNode {
    pub url: Url,
    pub group: String,
}
