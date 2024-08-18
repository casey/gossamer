use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Id([u8; Id::LEN]);

impl Id {
  pub const LEN: usize = 32;

  pub fn as_bytes(&self) -> &[u8; Self::LEN] {
    &self.0
  }
}

impl From<[u8; Id::LEN]> for Id {
  fn from(bytes: [u8; Id::LEN]) -> Self {
    Self(bytes)
  }
}

impl Display for Id {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", Hash::from(self.0))
  }
}

impl FromStr for Id {
  type Err = <blake3::Hash as FromStr>::Err;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    // todo: don't use hash
    Ok(Self(*blake3::Hash::from_str(s)?.as_bytes()))
  }
}

impl<'de> Deserialize<'de> for Id {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    Ok(Self(
      serde_bytes::ByteArray::<{ Id::LEN }>::deserialize(deserializer)?.into_array(),
    ))
  }
}

impl Serialize for Id {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serde_bytes::ByteArray::new(self.0).serialize(serializer)
  }
}
