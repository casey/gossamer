use super::*;

pub trait Cbor: Sized {
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
