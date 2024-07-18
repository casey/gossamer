use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Payload {
  FindNode(Hash),
  Nodes(Vec<Contact>),
  Ping,
  Pong,
  Store(Hash),
}
