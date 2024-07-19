use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub))]
pub(crate) enum Error {
  #[snafu(display("app package must be of type `app` not `{ty}`"))]
  AppType {
    backtrace: Option<Backtrace>,
    ty: Type,
  },
  #[snafu(display("failed to get current directory"))]
  CurrentDir {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("failed to deserialize YAML package metadata at `{path}`"))]
  DeserializeMetadata {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
    source: serde_yaml::Error,
  },
  #[snafu(display("missing `index.html` in `{root}`"))]
  Index {
    backtrace: Option<Backtrace>,
    root: Utf8PathBuf,
  },
  #[snafu(display("invalid page filename `{path}`"))]
  InvalidPage {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
    source: ParseIntError,
  },
  #[snafu(display("I/O error at `{path}`"))]
  Io {
    backtrace: Option<Backtrace>,
    path: Utf8PathBuf,
    source: io::Error,
  },
  #[snafu(display("missing `metadata.yaml` in `{root}`"))]
  MetadataMissing {
    backtrace: Option<Backtrace>,
    root: Utf8PathBuf,
  },
  #[snafu(display("failed to initialize node"))]
  NodeInitialize {
    backtrace: Option<Backtrace>,
    source: node::Error,
  },
  #[snafu(display("comic package in `{root}` contains no pages"))]
  NoPages {
    backtrace: Option<Backtrace>,
    root: Utf8PathBuf,
  },
  #[snafu(display("failed to open `{url}`"))]
  Open {
    backtrace: Option<Backtrace>,
    source: io::Error,
    url: String,
  },
  #[snafu(display("package output `{output}` may not be in `{root}`"))]
  OutputInRoot {
    backtrace: Option<Backtrace>,
    output: Utf8PathBuf,
    root: Utf8PathBuf,
  },
  #[snafu(display("package output `{output}` may not be a directory"))]
  OutputIsDir {
    backtrace: Option<Backtrace>,
    output: Utf8PathBuf,
  },
  #[snafu(display("failed to load package `{path}`"))]
  PackageLoad {
    path: Utf8PathBuf,
    #[snafu(backtrace)]
    source: package::Error,
  },
  #[snafu(display("failed to save package to `{path}`"))]
  PackageSave {
    path: Utf8PathBuf,
    #[snafu(backtrace)]
    source: package::Error,
  },
  #[snafu(display("multifple page {page}s"))]
  PageDuplicated {
    backtrace: Option<Backtrace>,
    page: u64,
  },
  #[snafu(display("page {page} missing"))]
  PageMissing {
    backtrace: Option<Backtrace>,
    page: u64,
  },
  #[snafu(
    display("path contains invalid UTF-8: `{}`", path.display())
  )]
  PathUnicode {
    backtrace: Option<Backtrace>,
    path: PathBuf,
  },
  #[snafu(display("I/O error initializing async runtime"))]
  Runtime {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("I/O error serving on {address}"))]
  Serve {
    address: SocketAddr,
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("unexpected file `{file}` in {ty} package"))]
  UnexpectedFile {
    backtrace: Option<Backtrace>,
    file: Utf8PathBuf,
    ty: Type,
  },
  #[snafu(display("failed to walk directory `{root}`"))]
  WalkDir {
    backtrace: Option<Backtrace>,
    root: Utf8PathBuf,
    source: walkdir::Error,
  },
}

impl Error {
  pub(crate) fn report(&self) {
    eprintln!("error: {self}");

    for (i, err) in self.iter_chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("because:");
      }

      eprintln!("- {err}");
    }

    if let Some(backtrace) = self.backtrace() {
      if backtrace.status() == BacktraceStatus::Captured {
        eprintln!();
        eprintln!("backtrace:");
        eprintln!("{backtrace}");
      }
    }
  }
}
