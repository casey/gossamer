use super::*;

#[derive(Copy, Clone, Debug, Deserialize, IntoStaticStr, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Type {
  App,
  Comic,
}

impl Type {
  fn name(self) -> &'static str {
    self.into()
  }
}

impl Display for Type {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}
