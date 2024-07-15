use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Node {
  pub address: IpAddr,
  pub directory: HashMap<Hash, HashSet<Contact>>,
  pub id: Hash,
  pub port: u16,
  pub received: u64,
  pub routing_table: Vec<Vec<Contact>>,
  pub sent: u64,
}
