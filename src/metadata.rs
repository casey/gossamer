use super::*;

#[derive(Copy, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Metadata {
  App { handles: Type },
  Comic,
}

impl Metadata {
  pub const PATH: &'static str = "metadata.yaml";

  pub fn template(self, root: &Utf8Path, paths: &HashSet<Utf8PathBuf>) -> Result<Template> {
    match self {
      Self::App { handles } => {
        if !paths.contains(Utf8Path::new("index.html")) {
          return Err(Error::Index {
            backtrace: Backtrace::capture(),
            root: root.into(),
          });
        };
        Ok(Template::App { handles })
      }
      Self::Comic => {
        let mut pages: Vec<(u64, Utf8PathBuf)> = Vec::new();

        let page_re = Regex::new(r"^(\d+)\.jpg$").unwrap();

        for path in paths {
          if path == Metadata::PATH {
            continue;
          }

          let captures = page_re
            .captures(path.as_ref())
            .context(error::UnexpectedFile {
              file: path.clone(),
              ty: self.ty(),
            })?;

          pages.push((
            captures[1].parse().context(error::InvalidPage { path })?,
            path.clone(),
          ));
        }

        pages.sort();

        for (i, (page, _path)) in pages.iter().enumerate() {
          let i = i.into_u64();
          let page = *page;

          if i < page {
            return Err(Error::PageMissing {
              backtrace: Backtrace::capture(),
              page: i,
            });
          }

          if i > page {
            return Err(Error::PageDuplicated {
              backtrace: Backtrace::capture(),
              page,
            });
          }
        }

        Ok(Template::Comic {
          pages: pages.into_iter().map(|(_page, path)| path).collect(),
        })
      }
    }
  }

  pub fn ty(self) -> Type {
    match self {
      Self::App { .. } => Type::App,
      Self::Comic => Type::Comic,
    }
  }
}
