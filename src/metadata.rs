use super::*;

#[derive(Copy, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Metadata {
  App { handles: Type },
  Comic,
}

impl Metadata {
  pub const PATH: &'static str = "metadata.yaml";

  pub fn ty(self) -> Type {
    match self {
      Self::App { .. } => Type::App,
      Self::Comic => Type::Comic,
    }
  }
}
