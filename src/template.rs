use super::*;

pub(crate) struct Template {
  pub(crate) name: String,
  pub(crate) media: Media,
}

pub(crate) enum Media {
  Comic { pages: Vec<Utf8PathBuf> },
}

impl Template {
  pub(crate) fn manifest(self, hashes: &HashMap<Utf8PathBuf, (Hash, u64)>) -> Manifest {
    let media = match self.media {
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
