use std::sync::{RwLock, Arc, Weak};
use tokio::time;

pub trait EdgeNodeProvider: Send + Sync {
    fn get_edge_nodes(&self, amount: usize) -> Vec<String>;
}

impl EdgeNodeProvider for Vec<String> {
    fn get_edge_nodes(&self, amount: usize) -> Vec<String> {
        let mut edge_nodes = Vec::with_capacity(amount);

        let mut inf_nodes = self.iter().cycle();

        for _ in 0..amount {
            edge_nodes.push(inf_nodes.next().unwrap().clone());
        }

        edge_nodes
    }
}

type EdgeNodeStorage = RwLock<Arc<Vec<String>>>;

pub struct UpdatingEdgeNodeProvider {
    edge_nodes: Arc<EdgeNodeStorage>,
}

impl UpdatingEdgeNodeProvider {
    pub fn new() -> Self {
        let edge_nodes = Arc::new(RwLock::new(Arc::new(vec![])));

        let provider = UpdatingEdgeNodeProvider {
            edge_nodes: Arc::clone(&edge_nodes),
        };

        start_update_edge_nodes_loop(Arc::downgrade(&edge_nodes));

        provider
    }

    fn current_edge_nodes(&self) -> Arc<Vec<String>> {
        Arc::clone(&self.edge_nodes.read().unwrap())
    }
}

fn start_update_edge_nodes_loop(edge_nodes: Weak<EdgeNodeStorage>) {
    tokio::spawn(async move {
        update_edge_nodes_loop(edge_nodes).await
    });
}

async fn update_edge_nodes_loop(edge_nodes: Weak<EdgeNodeStorage>) {
    let mut interval = time::interval(time::Duration::from_secs(1));

    let mut counter = 0;
    loop {
        interval.tick().await;

        let edge_nodes = match edge_nodes.upgrade() {
            Some(provider) => provider,
            None => {
                println!("Couldn't get reference to the edge node storage, ending update loop");
                break
            },
        };

        counter += 1;

        println!("updating edge nodes: {}", counter);

        let new_edge_nodes = vec![format!("http://{}.com", counter)];
        *edge_nodes.write().unwrap() = Arc::new(new_edge_nodes);
    }
}

impl EdgeNodeProvider for UpdatingEdgeNodeProvider {
    fn get_edge_nodes(&self, amount: usize) -> Vec<String> {
        let mut edge_nodes = Vec::with_capacity(amount);

        let current_edge_nodes = self.current_edge_nodes();
        let mut inf_nodes = current_edge_nodes.iter().cycle();

        for _ in 0..amount {
            edge_nodes.push(inf_nodes.next().unwrap().clone());
        }

        edge_nodes
    }
}
