use super::*;

pub struct Template {
  pub name: String,
  pub media: Media,
}

pub enum Media {
  App { target: Target },
  Comic { pages: Vec<Utf8PathBuf> },
}

impl Template {
  pub fn manifest(self, hashes: &HashMap<Utf8PathBuf, (Hash, u64)>) -> Manifest {
    let media = match self.media {
      Media::App { target } => {
        let mut paths = BTreeMap::new();

        for (path, (hash, _len)) in hashes {
          paths.insert(path.to_string(), *hash);
        }

        super::Media::App { target, paths }
      }
      Media::Comic { pages } => super::Media::Comic {
        pages: pages
          .into_iter()
          .map(|path| hashes.get(&path).unwrap().0)
          .collect(),
      },
    };

    Manifest {
      name: self.name,
      media,
    }
  }
}
