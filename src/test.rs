use super::*;

pub(crate) use {std::fs, tempfile::TempDir};

pub(crate) fn tempdir() -> TempDir {
  tempfile::tempdir().unwrap()
}

pub(crate) trait TempDirExt {
  fn path_utf8(&self) -> &Utf8Path;

  fn touch(&self, path: impl AsRef<Utf8Path>) {
    self.write(path, []);
  }

  fn write(&self, path: impl AsRef<Utf8Path>, data: impl AsRef<[u8]>);

  fn write_yaml(&self, path: impl AsRef<Utf8Path>, value: impl Serialize) {
    self.write(path, serde_yaml::to_string(&value).unwrap());
  }

  fn join(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
    self.path_utf8().join(path)
  }
}

impl TempDirExt for TempDir {
  fn path_utf8(&self) -> &Utf8Path {
    self.path().try_into().unwrap()
  }

  fn write(&self, path: impl AsRef<Utf8Path>, data: impl AsRef<[u8]>) {
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

pub(crate) static PACKAGES: Lazy<Packages> = Lazy::new(|| {
  let dir = tempdir();

  let comic = dir.join("comic.package");

  subcommand::package::Package {
    root: "tests/packages/comic".into(),
    output: comic.clone(),
  }
  .run()
  .unwrap();

  Packages {
    dir,
    comic: Package::load(&comic).unwrap(),
  }
});

pub(crate) struct Packages {
  comic: Package,
  #[allow(unused)]
  dir: TempDir,
}

impl Packages {
  pub(crate) fn comic(&self) -> &Package {
    &self.comic
  }
}
