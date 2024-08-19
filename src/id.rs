use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub(crate) struct Id([u8; Id::LEN]);

impl Id {
  pub(crate) const LEN: usize = 32;

  pub(crate) fn as_bytes(&self) -> &[u8; Self::LEN] {
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn serde() {
    let id = Id::from([
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
      26, 27, 28, 29, 30, 31,
    ]);

    let mut expected = Vec::new();
    expected.extend_from_slice(&[0x58, 32]);
    expected.extend_from_slice(id.as_bytes());

    assert_eq!(expected.len(), 34);

    assert_eq!(id.to_cbor(), expected);
    assert_eq!(Id::from_cbor(&expected).unwrap(), id);
  }
}
