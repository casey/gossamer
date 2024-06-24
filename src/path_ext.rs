use {super::*, std::path::Path};

pub trait PathExt {
  fn try_into_utf8<'a>(&'a self) -> Result<&'a Utf8Path>;
}

impl PathExt for Path {
  fn try_into_utf8<'a>(&'a self) -> Result<&'a Utf8Path> {
    Utf8Path::from_path(self).context(error::PathUnicode { path: self })
  }
}

pub trait PathBufExt {
  #[allow(unused)]
  fn try_into_utf8(self) -> Result<Utf8PathBuf>;
}

impl PathBufExt for PathBuf {
  fn try_into_utf8(self) -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(self).map_err(|path| Error::PathUnicode {
      path,
      backtrace: Backtrace::capture(),
    })
  }
}
