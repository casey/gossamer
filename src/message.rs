use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct Message {
  pub(crate) from: Hash,
  pub(crate) payload: Payload,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn size() {
    let from = Hash::bytes(&[]);
    assert!(
      Message {
        from,
        payload: Payload::Pong
      }
      .to_cbor()
      .len()
        <= MAX_UDP_PAYLOAD
    );
  }
}
