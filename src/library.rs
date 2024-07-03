use super::*;

#[derive(Default)]
pub struct Library {
  packages: HashMap<Hash, Package>,
}

impl Library {
  pub fn add(&mut self, package: Package) {
    self.packages.insert(package.hash, package);
  }

  pub fn package(&self, hash: Hash) -> Option<&Package> {
    self.packages.get(&hash)
  }
}
