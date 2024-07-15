use super::*;

#[derive(Debug)]
pub(crate) struct App {
  pub(crate) name: String,
  pub(crate) paths: BTreeMap<String, Hash>,
  pub(crate) target: Target,
}
