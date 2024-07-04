use super::*;

pub use {
  std::{fs, sync::Mutex},
  tempfile::TempDir,
};

pub fn tempdir() -> TempDir {
  tempfile::tempdir().unwrap()
}

pub trait TempDirExt {
  fn path_utf8(&self) -> &Utf8Path;
}

impl TempDirExt for TempDir {
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

static PACKAGES: Mutex<Option<TempDir>> = Mutex::new(None);

pub fn packages() -> Utf8PathBuf {
  let mut packages = PACKAGES.lock().unwrap();

  if packages.is_none() {
    let tempdir = tempdir();

    subcommand::package::Package {
      root: "apps/comic".into(),
      output: tempdir.path_utf8().join("app.package"),
    }
    .run()
    .unwrap();

    subcommand::package::Package {
      root: "apps/library".into(),
      output: tempdir.path_utf8().join("library.package"),
    }
    .run()
    .unwrap();

    subcommand::package::Package {
      root: "content/comic".into(),
      output: tempdir.path_utf8().join("content.package"),
    }
    .run()
    .unwrap();

    *packages = Some(tempdir);
  }

  packages.as_ref().unwrap().path_utf8().into()
}
