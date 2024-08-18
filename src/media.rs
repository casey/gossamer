use super::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Media {
  Comic { pages: Vec<Hash> },
}

impl Media {
  pub fn ty(&self) -> Type {
    match self {
      Self::Comic { .. } => Type::Comic,
    }
  }
}
