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
  regex::Regex,
  serde::{Deserialize, Serialize},
  snafu::{Backtrace, ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{
    backtrace::BacktraceStatus,
    collections::{BTreeMap, HashMap, HashSet},
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufReader, BufWriter, Cursor, Read, Write},
    net::SocketAddr,
    num::ParseIntError,
    path::PathBuf,
    process,
    sync::Arc,
  },
  walkdir::WalkDir,
};

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
    eprintln!("error: {err}");

    for (i, err) in err.iter_chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("because:");
      }

      eprintln!("- {err}");
    }

    if let Some(backtrace) = err.backtrace() {
      if backtrace.status() == BacktraceStatus::Captured {
        eprintln!("backtrace:");
        eprintln!("{backtrace}");
      }
    }

    process::exit(EXIT_FAILURE)
  }
}
