use super::*;

#[derive(Eq, PartialEq, Debug, Hash, Copy, Clone)]
pub struct Hash(blake3::Hash);

impl Hash {
  pub fn as_bytes(&self) -> &[u8; 32] {
    self.0.as_bytes()
  }

  #[allow(clippy::self_named_constructors)]
  pub fn bytes(input: &[u8]) -> Self {
    Self(blake3::hash(input))
  }

  pub fn reader(read: impl Read) -> io::Result<Self> {
    let mut hasher = blake3::Hasher::new();

    hasher.update_reader(read)?;

    Ok(Self(hasher.finalize()))
  }
}

impl Ord for Hash {
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_bytes().cmp(other.as_bytes())
  }
}

impl PartialOrd for Hash {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Display for Hash {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<[u8; 32]> for Hash {
  fn from(bytes: [u8; 32]) -> Self {
    Self(bytes.into())
  }
}

impl FromStr for Hash {
  type Err = <blake3::Hash as FromStr>::Err;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(blake3::Hash::from_str(s)?))
  }
}

impl<'de> Deserialize<'de> for Hash {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    Ok(Self(
      serde_bytes::ByteArray::<32>::deserialize(deserializer)?
        .into_array()
        .into(),
    ))
  }
}

impl Serialize for Hash {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serde_bytes::ByteArray::new(*self.as_bytes()).serialize(serializer)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  trait Cbor: Sized {
    fn to_cbor(&self) -> Vec<u8>;
    fn from_cbor(cbor: &[u8]) -> Result<Self, ciborium::de::Error<io::Error>>;
  }

  impl<T: DeserializeOwned + Serialize> Cbor for T {
    fn to_cbor(&self) -> Vec<u8> {
      let mut buffer = Vec::new();
      ciborium::into_writer(self, &mut buffer).unwrap();
      buffer
    }

    fn from_cbor(cbor: &[u8]) -> Result<Self, ciborium::de::Error<io::Error>> {
      ciborium::from_reader(Cursor::new(cbor))
    }
  }

  #[test]
  fn serde() {
    let hash = Hash::bytes(&[]);

    let mut expected = Vec::new();
    expected.extend_from_slice(&[0x58, 32]);
    expected.extend_from_slice(hash.as_bytes());

    assert_eq!(expected.len(), 34);

    assert_eq!(hash.to_cbor(), expected);
    assert_eq!(Hash::from_cbor(&expected).unwrap(), hash);
  }
}
