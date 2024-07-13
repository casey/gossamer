use super::*;

pub use {std::fs, tempfile::TempDir};

pub fn tempdir() -> TempDir {
  tempfile::tempdir().unwrap()
}

pub trait TempDirExt {
  fn path_utf8(&self) -> &Utf8Path;

  fn write(&self, path: &str, data: impl AsRef<[u8]>);

  fn join(&self, path: &str) -> Utf8PathBuf {
    self.path_utf8().join(path)
  }
}

impl TempDirExt for TempDir {
  fn path_utf8(&self) -> &Utf8Path {
    self.path().try_into().unwrap()
  }

  fn write(&self, path: &str, data: impl AsRef<[u8]>) {
    let path = self.path_utf8().join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, data).unwrap();
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

  let root = path.join("root.package");

  let comic = path.join("comic.package");

  subcommand::package::Package {
    root: "tests/packages/app-comic".into(),
    output: app.clone(),
  }
  .run()
  .unwrap();

  subcommand::package::Package {
    root: "tests/packages/app-root".into(),
    output: root.clone(),
  }
  .run()
  .unwrap();

  subcommand::package::Package {
    root: "tests/packages/comic".into(),
    output: comic.clone(),
  }
  .run()
  .unwrap();

  Packages {
    dir,
    app: Package::load(&app).unwrap(),
    root: Package::load(&root).unwrap(),
    comic: Package::load(&comic).unwrap(),
  }
});

pub struct Packages {
  root: Package,
  app: Package,
  comic: Package,
  #[allow(unused)]
  dir: TempDir,
}

impl Packages {
  pub fn app(&self) -> &Package {
    &self.app
  }

  pub fn root(&self) -> &Package {
    &self.root
  }

  pub fn comic(&self) -> &Package {
    &self.comic
  }
}
