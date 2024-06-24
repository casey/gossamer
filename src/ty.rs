use super::*;

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
  App,
  Comic,
}

impl Display for Type {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::App => write!(f, "app"),
      Self::Comic => write!(f, "comic"),
    }
  }
}
