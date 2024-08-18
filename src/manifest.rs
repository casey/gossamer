use super::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Manifest {
  pub name: String,
  pub media: Media,
}

impl Manifest {
  pub fn ty(&self) -> Type {
    self.media.ty()
  }
}
