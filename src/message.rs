use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct Message {
  pub(crate) from: Hash,
  pub(crate) payload: Payload,
}

impl Message {
  pub(crate) const MAX_SIZE: usize = 2048;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn size() {
    let from = Hash::bytes(&[]);

    for payload in [
      Payload::Pong,
      Payload::Pong,
      Payload::Nodes(vec![
        Contact {
          address: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(),
          port: 0,
          id: Hash::bytes(&[]),
        };
        K
      ]),
    ] {
      let message = Message { from, payload }.to_cbor();
      let len = message.len();
      assert!(
        len <= Message::MAX_SIZE,
        "message length {len} over {}",
        Message::MAX_SIZE,
      );
    }
  }
}
