use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Payload {
  Ping,
  Pong,
  Store(Hash),
}
