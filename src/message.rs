use super::*;

#[derive(Debug, Deserialize, Serialize, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Message {
  FindNode(Hash),
  Nodes(Vec<Peer>),
  Ping,
  Pong,
  Store(Hash),
}
