use super::*;

#[derive(Debug, Deserialize, Serialize, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Message {
  Get(Hash),
  Ping,
  Search,
}
