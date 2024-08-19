#![allow(clippy::result_large_err)]

use {
  self::{
    deserialize_from_str::DeserializeFromStr, error::Error, from_cbor::FromCbor, hash::Hash,
    id::Id, into_u64::IntoU64, manifest::Manifest, media::Media, message::Message,
    metadata::Metadata, node::Node, package::Package, path_ext::PathExt, peer::Peer,
    read_ext::ReadExt, report::Report, subcommand::Subcommand, template::Template, to_cbor::ToCbor,
    ty::Type, write_ext::WriteExt,
  },
  axum::{body::Body, http::header},
  boilerplate::Boilerplate,
  camino::{Utf8Path, Utf8PathBuf},
  clap::Parser,
  html_escaper::{Escape, Trusted},
  libc::EXIT_FAILURE,
  mime_guess::{mime, Mime},
  quinn::{Connection, Endpoint, Incoming, RecvStream, SendStream},
  rand::Rng,
  regex::Regex,
  regex_static::{lazy_regex, once_cell::sync::Lazy},
  serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer},
  snafu::{ensure, ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{
    any::Any,
    backtrace::{Backtrace, BacktraceStatus},
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::{self, Display, Formatter},
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
mod from_cbor;
mod hash;
mod id;
mod into_u64;
mod manifest;
mod media;
mod message;
mod metadata;
mod node;
mod package;
mod passthrough;
mod path_ext;
mod peer;
mod read_ext;
mod report;
mod response;
mod subcommand;
mod template;
mod to_cbor;
mod ty;
mod write_ext;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(err) = Subcommand::parse().run() {
    err.report();
    process::exit(EXIT_FAILURE)
  }
}
