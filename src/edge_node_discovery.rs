pub trait EdgeNodeProvider {
    fn get_edge_nodes(&self, amount: usize) -> Vec<String>;
}
