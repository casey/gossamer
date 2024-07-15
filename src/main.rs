#![allow(clippy::result_large_err)]

use {
  self::{
    deserialize_from_str::DeserializeFromStr, distance::Distance, error::Error, into_u64::IntoU64,
    library::Library, message::Message, metadata::Metadata, node::Node, package::Package,
    path_ext::PathExt, payload::Payload, read_ext::ReadExt, subcommand::Subcommand,
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
  media::{Cbor, Contact, Hash, Manifest, Media, Target, Type},
  mime_guess::{mime, Mime},
  rand::Rng,
  regex::Regex,
  regex_static::{lazy_regex, once_cell::sync::Lazy},
  serde::{Deserialize, Deserializer, Serialize},
  snafu::{ensure, ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{
    backtrace::{Backtrace, BacktraceStatus},
    cmp::Ordering,
    collections::{BTreeMap, HashMap, HashSet},
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
  tokio::{net::UdpSocket, sync::RwLock},
  walkdir::WalkDir,
};

// Maximum UDP packet size deliverable without fragmentation.
//
// Equal to `MIN_MTU - MAX_IP_HEADER - MAX_UDP_HEADER`.
const MAX_UDP_PAYLOAD: usize = 508 - 60 - 8;

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
mod path_ext;
mod payload;
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
