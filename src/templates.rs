use super::*;

#[derive(Boilerplate)]
#[boilerplate(filename = "root.html")]
pub(crate) struct Root {
  pub(crate) node: Option<media::api::Node>,
  pub(crate) package: Option<Hash>,
  pub(crate) library: Arc<Library>,
  pub(crate) peer: Option<(Id, BTreeMap<Hash, Manifest>)>,
}
