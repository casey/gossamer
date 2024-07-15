use super::*;

#[derive(Default)]
pub(crate) struct Library {
  pub(crate) bookmarks: Mutex<BTreeSet<Hash>>,
  pub(crate) handlers: BTreeMap<Target, Hash>,
  pub(crate) packages: BTreeMap<Hash, Package>,
}

impl Library {
  pub(crate) fn add(&mut self, package: Package) {
    if let Media::App { target, .. } = &package.manifest.media {
      self.handlers.insert(*target, package.hash);
    }

    self.packages.insert(package.hash, package);
  }

  pub(crate) fn package(&self, hash: Hash) -> Option<&Package> {
    self.packages.get(&hash)
  }

  pub(crate) fn packages(&self) -> &BTreeMap<Hash, Package> {
    &self.packages
  }

  pub(crate) fn handler(&self, target: Target) -> Option<&Package> {
    self
      .handlers
      .get(&target)
      .map(|hash| self.packages.get(hash).unwrap())
  }

  pub(crate) fn handlers(&self) -> &BTreeMap<Target, Hash> {
    &self.handlers
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn add() {
    let mut library = Library::default();

    let package = PACKAGES.app();

    library.add(package.clone());

    assert_eq!(library.package(package.hash), Some(package));
    assert_eq!(library.handler(Target::Comic), Some(package));
  }
}
