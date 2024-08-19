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
