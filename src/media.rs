use super::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(crate) enum Media {
  Comic { pages: Vec<Hash> },
}
