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

pub struct UpdatingEdgeNodeProvider {
    edge_nodes: RwLock<Arc<Vec<String>>>,
}

impl UpdatingEdgeNodeProvider {
    pub fn new() -> Arc<Self> {
        let provider = Arc::new(UpdatingEdgeNodeProvider {
            edge_nodes: RwLock::new(Arc::new(vec![])),
        });

        start_update_edge_nodes_loop(Arc::downgrade(&provider));

        provider
    }

    fn current_edge_nodes(&self) -> Arc<Vec<String>> {
        Arc::clone(&self.edge_nodes.read().unwrap())
    }
}

fn start_update_edge_nodes_loop(provider: Weak<UpdatingEdgeNodeProvider>) {
    tokio::spawn(async move {
        update_edge_nodes_loop(provider).await
    });
}

async fn update_edge_nodes_loop(provider: Weak<UpdatingEdgeNodeProvider>) {
    let mut interval = time::interval(time::Duration::from_secs(1));

    let mut counter = 0;
    loop {
        interval.tick().await;

        let provider = match provider.upgrade() {
            Some(provider) => provider,
            None => {
                println!("Couldn't get reference to the edge node provider, ending update loop");
                break
            },
        };

        counter += 1;

        println!("updating edge nodes: {}", counter);

        let new_edge_nodes = vec![format!("http://{}.com", counter)];
        *provider.edge_nodes.write().unwrap() = Arc::new(new_edge_nodes);
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
