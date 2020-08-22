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
