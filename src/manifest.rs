use super::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct Manifest {
  pub name: String,
  pub media: Media,
}
