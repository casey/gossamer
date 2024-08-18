use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Node {
  pub local: BTreeSet<Id>,
  // todo: this is misleading, if it's unspecified, then can't connect
  pub peer: Peer,
  pub received: u64,
  pub sent: u64,
}
