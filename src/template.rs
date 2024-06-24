use super::*;

pub enum Template {
  App,
  Comic { pages: Vec<Utf8PathBuf> },
}
