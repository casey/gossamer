use super::*;

pub(crate) trait ToCbor: Sized {
  fn to_cbor(&self) -> Vec<u8>;
}

impl<T: Serialize> ToCbor for T {
  fn to_cbor(&self) -> Vec<u8> {
    let mut buffer = Vec::new();
    ciborium::into_writer(self, &mut buffer).unwrap();
    buffer
  }
}

pub(crate) trait FromCbor: Sized {
  fn from_cbor(cbor: &[u8]) -> Result<Self, ciborium::de::Error<io::Error>>;
}

impl<T: DeserializeOwned> FromCbor for T {
  fn from_cbor(cbor: &[u8]) -> Result<Self, ciborium::de::Error<io::Error>> {
    ciborium::from_reader(Cursor::new(cbor))
  }
}
