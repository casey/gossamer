#![allow(clippy::result_large_err)]

use {
  self::{
    error::Error, into_u64::IntoU64, manifest::Manifest, metadata::Metadata, package::Package,
    path_ext::PathExt, read_ext::ReadExt, subcommand::Subcommand, template::Template, ty::Type,
    write_ext::WriteExt,
  },
  axum::http::header,
  blake3::{Hash, Hasher},
  camino::{Utf8Path, Utf8PathBuf},
  clap::Parser,
  libc::EXIT_FAILURE,
  mime_guess::{mime, Mime},
  regex::Regex,
  serde::{Deserialize, Serialize},
  snafu::{ensure, Backtrace, ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{
    backtrace::BacktraceStatus,
    collections::{BTreeMap, HashMap, HashSet},
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{self, BufReader, BufWriter, Cursor, Read, Seek, Write},
    net::SocketAddr,
    num::{ParseIntError, TryFromIntError},
    path::PathBuf,
    process,
    sync::{Arc, Mutex},
  },
  tempfile::TempDir,
  walkdir::WalkDir,
};

#[cfg(test)]
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

fn tempdir() -> tempfile::TempDir {
  tempfile::tempdir().unwrap()
}

trait TempDirExt {
  fn path_utf8(&self) -> &Utf8Path;
}

impl TempDirExt for tempfile::TempDir {
  fn path_utf8(&self) -> &Utf8Path {
    self.path().try_into().unwrap()
  }
}

mod error;
mod into_u64;
mod manifest;
mod metadata;
mod package;
mod path_ext;
mod read_ext;
mod subcommand;
mod template;
mod ty;
mod write_ext;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(err) = Subcommand::parse().run() {
    err.report();
    process::exit(EXIT_FAILURE)
  }
}
