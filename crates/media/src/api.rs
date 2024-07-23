use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Node {
  pub peer: Peer,
  pub directory: HashMap<Hash, HashSet<Peer>>,
  pub received: u64,
  pub routing_table: Vec<Vec<Peer>>,
  pub sent: u64,
}
