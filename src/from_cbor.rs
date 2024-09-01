use super::*;

pub(crate) trait FromCbor: Sized {
  fn from_cbor(cbor: &[u8]) -> Result<Self, ciborium::de::Error<io::Error>>;
}

impl<T: DeserializeOwned> FromCbor for T {
  fn from_cbor(cbor: &[u8]) -> Result<Self, ciborium::de::Error<io::Error>> {
    ciborium::from_reader(Cursor::new(cbor))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_cbor() {
    assert_eq!(u32::from_cbor(&[0b000_01010]).unwrap(), 10);
  }
}
