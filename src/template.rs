use super::*;

pub enum Template {
  App { target: Target },
  Comic { pages: Vec<Utf8PathBuf> },
}

impl Template {
  pub fn manifest(self, hashes: &HashMap<Utf8PathBuf, (Hash, u64)>) -> Manifest {
    match self {
      Self::App { target } => {
        let mut paths = BTreeMap::new();

        for (path, (hash, _len)) in hashes {
          paths.insert(path.to_string(), *hash);
        }

        Manifest::App { target, paths }
      }
      Self::Comic { pages } => Manifest::Comic {
        pages: pages
          .into_iter()
          .map(|path| hashes.get(&path).unwrap().0)
          .collect(),
      },
    }
  }
}
