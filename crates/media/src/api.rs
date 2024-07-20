use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Node {
  pub contact: Contact,
  pub directory: HashMap<Hash, HashSet<Contact>>,
  pub received: u64,
  pub routing_table: Vec<Vec<Contact>>,
  pub sent: u64,
}
