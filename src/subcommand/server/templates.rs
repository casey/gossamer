use super::*;

#[derive(Boilerplate)]
pub(crate) struct PageHtml<T: Display> {
  pub(crate) packages: Arc<BTreeMap<Hash, Package>>,
  pub(crate) main: T,
}

#[derive(Boilerplate)]
pub(crate) struct NodeHtml {
  pub(crate) local: BTreeSet<Id>,
  pub(crate) peer: Peer,
  pub(crate) received: u64,
  pub(crate) sent: u64,
}

#[derive(Boilerplate)]
pub(crate) struct PackageHtml {
  pub(crate) package: Arc<Package>,
}

#[derive(Boilerplate)]
pub(crate) struct SearchHtml {
  pub(crate) peer: Id,
  pub(crate) manifests: BTreeMap<Hash, Manifest>,
}
