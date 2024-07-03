use super::*;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Target {
  App,
  Comic,
  Library,
}

impl Display for Target {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::App => write!(f, "app"),
      Self::Comic => write!(f, "comic"),
      Self::Library => write!(f, "library"),
    }
  }
}
