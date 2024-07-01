use {super::*, std::path::Path};

pub trait PathExt {
  fn try_into_utf8(&self) -> Result<&Utf8Path>;
}

impl PathExt for Path {
  fn try_into_utf8(&self) -> Result<&Utf8Path> {
    Utf8Path::from_path(self).context(error::PathUnicode { path: self })
  }
}

pub trait PathBufExt {
  #[allow(unused)]
  fn try_into_utf8(self) -> Result<Utf8PathBuf>;
}

impl PathBufExt for PathBuf {
  fn try_into_utf8(self) -> Result<Utf8PathBuf> {
    match Utf8PathBuf::from_path_buf(self) {
      Ok(path) => Ok(path),
      Err(path) => Err(error::PathUnicode { path }.build()),
    }
  }
}
