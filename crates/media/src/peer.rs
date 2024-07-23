use super::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Peer {
  pub id: Hash,
  pub ip: IpAddr,
  pub port: u16,
}

impl Peer {
  pub fn socket_addr(self) -> SocketAddr {
    (self.ip, self.port).into()
  }
}

impl Display for Peer {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}@{}", self.id, self.socket_addr())
  }
}

impl FromStr for Peer {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    static RE: Lazy<Regex> = lazy_regex!("^(.*)@(.*)$");

    let captures = RE.captures(s).unwrap();

    let socket_addr = captures[2].parse::<SocketAddr>().unwrap();

    Ok(Self {
      id: captures[1].parse().unwrap(),
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
}
