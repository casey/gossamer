use super::*;

pub fn tempdir() -> tempfile::TempDir {
  tempfile::tempdir().unwrap()
}

pub trait TempDirExt {
  fn path_utf8(&self) -> &Utf8Path;
}

impl TempDirExt for tempfile::TempDir {
  fn path_utf8(&self) -> &Utf8Path {
    self.path().try_into().unwrap()
  }
}

macro_rules! assert_matches {
  ($expression:expr, $( $pattern:pat_param )|+ $( if $guard:expr )? $(,)?) => {
    match $expression {
      $( $pattern )|+ $( if $guard )? => {}
      left => panic!(
        "assertion failed: (left ~= right)\n  left: `{:?}`\n right: `{}`",
        left,
        stringify!($($pattern)|+ $(if $guard)?)
      ),
    }
  }
}
