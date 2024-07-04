use super::*;

pub use {std::fs, tempfile::TempDir};

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

pub static PACKAGES: Lazy<Packages> = Lazy::new(|| {
  let dir = tempdir();

  let path = dir.path_utf8();

  let app = path.join("app.package");

  let library = path.join("library.package");

  let comic = path.join("comic.package");

  subcommand::package::Package {
    root: "apps/comic".into(),
    output: app.clone(),
  }
  .run()
  .unwrap();

  subcommand::package::Package {
    root: "apps/library".into(),
    output: library.clone(),
  }
  .run()
  .unwrap();

  subcommand::package::Package {
    root: "content/comic".into(),
    output: comic.clone(),
  }
  .run()
  .unwrap();

  Packages {
    dir,
    app: Package::load(&app).unwrap(),
    library: Package::load(&library).unwrap(),
    comic: Package::load(&comic).unwrap(),
  }
});

pub struct Packages {
  library: Package,
  app: Package,
  comic: Package,
  #[allow(unused)]
  dir: TempDir,
}

impl Packages {
  pub fn app(&self) -> &Package {
    &self.app
  }

  pub fn library(&self) -> &Package {
    &self.library
  }

  pub fn comic(&self) -> &Package {
    &self.comic
  }
}
