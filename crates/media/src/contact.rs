use super::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Contact {
  pub address: IpAddr,
  pub port: u16,
  pub id: Hash,
}

impl Display for Contact {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}@{}",
      self.id,
      SocketAddr::from((self.address, self.port))
    )
  }
}

impl FromStr for Contact {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    static RE: Lazy<Regex> = lazy_regex!("^(.*)@(.*)$");

    let captures = RE.captures(s).unwrap();

    let socket_addr = captures[2].parse::<SocketAddr>().unwrap();

    Ok(Self {
      address: socket_addr.ip(),
      port: socket_addr.port(),
      id: captures[1].parse().unwrap(),
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
    fn case(s: &str, contact: Contact) {
      assert_eq!(s.parse::<Contact>().unwrap(), contact);
      assert_eq!(contact.to_string(), s);
    }

    case(
      "0f89ce0b671f7277b105035ea88341a81c5fceaed092eab29721fe6f86807133@1.2.3.4:5",
      Contact {
        address: Ipv4Addr::new(1, 2, 3, 4).into(),
        port: 5,
        id: "0f89ce0b671f7277b105035ea88341a81c5fceaed092eab29721fe6f86807133"
          .parse()
          .unwrap(),
      },
    );

    case(
      "0f89ce0b671f7277b105035ea88341a81c5fceaed092eab29721fe6f86807133@[1:2:3:4:5:6:7:8]:5",
      Contact {
        address: Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8).into(),
        port: 5,
        id: "0f89ce0b671f7277b105035ea88341a81c5fceaed092eab29721fe6f86807133"
          .parse()
          .unwrap(),
      },
    );
  }
}
