use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Manifest {
  App {
    handles: Type,
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
