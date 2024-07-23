#![allow(clippy::result_large_err)]

use {
  self::{
    deserialize_from_str::DeserializeFromStr, distance::Distance, error::Error, into_u64::IntoU64,
    library::Library, message::Message, metadata::Metadata, node::Node, package::Package,
    path_ext::PathExt, read_ext::ReadExt, report::Report, subcommand::Subcommand,
    template::Template, write_ext::WriteExt,
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
  media::{FromCbor, Hash, Manifest, Media, Peer, Target, ToCbor, Type},
  mime_guess::{mime, Mime},
  quinn::{Connection, Endpoint, Incoming, RecvStream, SendStream},
  rand::Rng,
  regex::Regex,
  regex_static::{lazy_regex, once_cell::sync::Lazy},
  serde::{Deserialize, Deserializer, Serialize},
  snafu::{ensure, ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{
    any::Any,
    backtrace::{Backtrace, BacktraceStatus},
    cmp::Ordering,
    collections::{BTreeMap, BinaryHeap, HashMap, HashSet},
    fmt::Display,
    fs::File,
    io::{self, BufReader, BufWriter, Cursor, Read, Seek, Write},
    iter,
    net::{IpAddr, SocketAddr},
    num::{ParseIntError, TryFromIntError},
    path::PathBuf,
    process, str,
    str::FromStr,
    sync::{
      atomic::{self, AtomicU64},
      Arc,
    },
  },
  strum::IntoStaticStr,
  tokio::sync::RwLock,
  walkdir::WalkDir,
};

// Maximum size of each routing table bucket, as well as the maximum number of
// nodes returned from FIND_NODE and FIND_VALUE query.
const K: usize = 20;

#[cfg(test)]
#[macro_use]
mod test;

#[cfg(test)]
use test::*;

mod deserialize_from_str;
mod distance;
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
