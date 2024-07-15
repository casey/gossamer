#![allow(clippy::result_large_err)]

use {
  self::{
    deserialize_from_str::DeserializeFromStr, error::Error, into_u64::IntoU64, library::Library,
    message::Message, metadata::Metadata, node::Node, package::Package, path_ext::PathExt,
    read_ext::ReadExt, report::Report, subcommand::Subcommand, template::Template,
    write_ext::WriteExt,
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
  media::{FromCbor, Hash, Id, Manifest, Media, Peer, Target, ToCbor, Type},
  mime_guess::{mime, Mime},
  quinn::{Connection, Endpoint, Incoming, RecvStream, SendStream},
  rand::Rng,
  regex::Regex,
  regex_static::{lazy_regex, once_cell::sync::Lazy},
  serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize},
  snafu::{ensure, ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{
    any::Any,
    backtrace::{Backtrace, BacktraceStatus},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::Display,
    fs::File,
    io::{self, BufReader, BufWriter, Cursor, Read, Seek, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    num::{ParseIntError, TryFromIntError},
    ops::{Deref, DerefMut},
    path::PathBuf,
    process, str,
    str::FromStr,
    sync::{
      atomic::{self, AtomicU64},
      Arc,
    },
    time::Duration,
  },
  strum::IntoStaticStr,
  tokio::sync::RwLock,
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
mod message;
mod metadata;
mod node;
mod package;
mod passthrough;
mod path_ext;
mod read_ext;
mod report;
mod response;
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
