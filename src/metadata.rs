use super::*;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct Metadata {
  pub(crate) name: String,
  pub(crate) media: Media,
}

impl Metadata {
  pub(crate) const PATH: &'static str = "metadata.yaml";

  pub(crate) fn load(path: &Utf8Path) -> Result<Self> {
    serde_yaml::from_reader(&File::open(path).context(error::Io { path })?)
      .context(error::DeserializeMetadata { path })
  }

  pub(crate) fn template(self, root: &Utf8Path, paths: &HashSet<Utf8PathBuf>) -> Result<Template> {
    let media = match &self.media {
      Media::App { target } => {
        ensure!(
          paths.contains(Utf8Path::new("index.html")),
          error::Index { root }
        );
        template::Media::App { target: *target }
      }
      Media::Comic => {
        let mut pages: Vec<(u64, Utf8PathBuf)> = Vec::new();

        static PAGE: Lazy<Regex> = lazy_regex!(r"^(\d+)\.jpg$");

        for path in paths {
          let captures = PAGE
            .captures(path.as_ref())
            .context(error::UnexpectedFile {
              file: path.clone(),
              ty: self.media.ty(),
            })?;

          pages.push((
            captures[1].parse().context(error::InvalidPage { path })?,
            path.clone(),
          ));
        }

        ensure!(!pages.is_empty(), error::NoPages { root });

        pages.sort();

        for (i, (page, _path)) in pages.iter().enumerate() {
          let i = i.into_u64();
          let page = *page;

          ensure!(i >= page, error::PageMissing { page: i });
          ensure!(i <= page, error::PageDuplicated { page });
        }

        template::Media::Comic {
          pages: pages.into_iter().map(|(_page, path)| path).collect(),
        }
      }
    };

    Ok(Template {
      media,
      name: self.name.clone(),
    })
  }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(crate) enum Media {
  App { target: Target },
  Comic,
}

impl Media {
  pub(crate) fn ty(&self) -> Type {
    match self {
      Self::App { .. } => Type::App,
      Self::Comic => Type::Comic,
    }
  }
}
