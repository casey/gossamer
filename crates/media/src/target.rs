use super::*;

#[derive(
  Copy, Clone, Debug, Deserialize, IntoStaticStr, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Target {
  App,
  Comic,
  Root,
}

impl Target {
  fn name(self) -> &'static str {
    self.into()
  }
}

impl Display for Target {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}

impl From<Type> for Target {
  fn from(ty: Type) -> Self {
    match ty {
      Type::App => Self::App,
      Type::Comic => Self::Comic,
    }
  }
}
