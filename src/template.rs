use super::*;

pub enum Template {
  App { handles: Type },
  Comic { pages: Vec<Utf8PathBuf> },
}
