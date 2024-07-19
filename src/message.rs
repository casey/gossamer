use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct Message {
  pub(crate) from: Hash,
  pub(crate) payload: Payload,
}
