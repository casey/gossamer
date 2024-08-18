use super::*;

#[derive(Boilerplate)]
#[boilerplate(filename = "root.html")]
pub(crate) struct Root {
  pub(crate) node: Option<api::Node>,
  pub(crate) package: Option<Hash>,
  pub(crate) packages: Arc<BTreeMap<Hash, Package>>,
  pub(crate) peer: Option<(Id, BTreeMap<Hash, Manifest>)>,
}
