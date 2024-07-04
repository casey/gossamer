use super::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Manifest {
  App {
    target: Target,
    paths: BTreeMap<String, Hash>,
  },
  Comic {
    pages: Vec<Hash>,
  },
}

impl Manifest {
  pub fn ty(&self) -> Type {
    match self {
      Self::App { .. } => Type::App,
      Self::Comic { .. } => Type::Comic,
    }
  }
}
