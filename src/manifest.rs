use super::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct Manifest {
  pub(crate) name: String,
  pub(crate) media: Media,
}
