use super::*;

#[derive(Deserialize)]
pub struct Metadata {
  #[serde(rename = "type")]
  pub ty: Type,
}

impl Metadata {
  pub const PATH: &'static str = "metadata.yaml";
}
