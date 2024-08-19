use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub(crate) enum Error {
  #[snafu(display("invalid peer address `{input}`"))]
  Address {
    input: String,
    source: AddrParseError,
  },
  #[snafu(display("invalid peer ID `{input}`"))]
  Id {
    input: String,
    source: blake3::HexError,
  },
  #[snafu(display("invalid peer `{input}`"))]
  Invalid { input: String },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, Ord, PartialOrd)]
pub(crate) struct Peer {
  pub(crate) id: Id,
  pub(crate) ip: IpAddr,
  pub(crate) port: u16,
}

impl Peer {
  pub(crate) fn socket_addr(self) -> SocketAddr {
    (self.ip, self.port).into()
  }
}

impl Display for Peer {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}@{}", self.id, self.socket_addr())
  }
}

impl FromStr for Peer {
  type Err = Error;

  fn from_str(input: &str) -> Result<Self, Self::Err> {
    static RE: Lazy<Regex> = lazy_regex!("^(.*)@(.*)$");

    let captures = RE.captures(input).context(InvalidError { input })?;

    let socket_addr = captures[2]
      .parse::<SocketAddr>()
      .context(AddressError { input })?;

    Ok(Self {
      id: captures[1].parse().context(IdError { input })?,
      ip: socket_addr.ip(),
      port: socket_addr.port(),
    })
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    std::net::{Ipv4Addr, Ipv6Addr},
  };

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, peer: Peer) {
      assert_eq!(s.parse::<Peer>().unwrap(), peer);
      assert_eq!(peer.to_string(), s);
    }

    case(
      "0f89ce0b671f7277b105035ea88341a81c5fceaed092eab29721fe6f86807133@1.2.3.4:5",
      Peer {
        port: 5,
        id: "0f89ce0b671f7277b105035ea88341a81c5fceaed092eab29721fe6f86807133"
          .parse()
          .unwrap(),
        ip: Ipv4Addr::new(1, 2, 3, 4).into(),
      },
    );

    case(
      "0f89ce0b671f7277b105035ea88341a81c5fceaed092eab29721fe6f86807133@[1:2:3:4:5:6:7:8]:5",
      Peer {
        port: 5,
        id: "0f89ce0b671f7277b105035ea88341a81c5fceaed092eab29721fe6f86807133"
          .parse()
          .unwrap(),
        ip: Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8).into(),
      },
    );
  }

  #[test]
  fn serde() {
    let peer = Peer {
      port: 5,
      id: "0f89ce0b671f7277b105035ea88341a81c5fceaed092eab29721fe6f86807133"
        .parse()
        .unwrap(),
      ip: Ipv4Addr::new(1, 2, 3, 4).into(),
    };

    let cbor = peer.to_cbor();

    assert_eq!(Peer::from_cbor(&cbor).unwrap(), peer);
  }
}
