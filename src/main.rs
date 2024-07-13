#![allow(clippy::result_large_err)]

use {
  self::{
    deserialize_from_str::DeserializeFromStr, error::Error, into_u64::IntoU64, library::Library,
    metadata::Metadata, package::Package, path_ext::PathExt, read_ext::ReadExt,
    subcommand::Subcommand, template::Template, write_ext::WriteExt,
  },
  axum::{
    body::Body,
    http::{
      self,
      header::{self, HeaderValue},
      Uri,
    },
  },
  camino::{Utf8Path, Utf8PathBuf},
  clap::Parser,
  libc::EXIT_FAILURE,
  media::{Cbor, Hash, Manifest, Media, Target, Type},
  mime_guess::{mime, Mime},
  regex::Regex,
  regex_static::{lazy_regex, once_cell::sync::Lazy},
  serde::{Deserialize, Deserializer, Serialize},
  snafu::{ensure, ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{
    backtrace::{Backtrace, BacktraceStatus},
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
    fs::File,
    io::{self, BufReader, BufWriter, Cursor, Read, Seek, Write},
    net::SocketAddr,
    num::{ParseIntError, TryFromIntError},
    path::PathBuf,
    process, str,
    str::FromStr,
    sync::Arc,
  },
  walkdir::WalkDir,
};

#[cfg(test)]
#[macro_use]
mod test;

#[cfg(test)]
use test::*;

mod deserialize_from_str;
mod error;
mod into_u64;
mod library;
mod metadata;
mod package;
mod path_ext;
mod read_ext;
mod subcommand;
mod template;
mod write_ext;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(err) = Subcommand::parse().run() {
    err.report();
    process::exit(EXIT_FAILURE)
  }
}
