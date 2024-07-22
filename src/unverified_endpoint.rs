use {
  super::*,
  bytes::BytesMut,
  quinn::{
    crypto::{
      self, AeadKey, CryptoError, ExportKeyingMaterialError, HandshakeTokenKey, HeaderKey, KeyPair,
      Keys, PacketKey, Session, UnsupportedVersion,
    },
    ClientConfig, ConnectError, Endpoint, ServerConfig,
  },
  quinn_proto::{transport_parameters::TransportParameters, ConnectionId, Side, TransportError},
};

struct PassthroughKey;

impl PassthroughKey {
  fn keys() -> Keys {
    Keys {
      header: KeyPair {
        local: Box::new(PassthroughKey),
        remote: Box::new(PassthroughKey),
      },
      packet: KeyPair {
        local: Box::new(PassthroughKey),
        remote: Box::new(PassthroughKey),
      },
    }
  }
}

impl HandshakeTokenKey for PassthroughKey {
  fn aead_from_hkdf(&self, _random_bytes: &[u8]) -> Box<dyn AeadKey> {
    todo!()
  }
}

impl AeadKey for PassthroughKey {
  fn seal(&self, _data: &mut Vec<u8>, _additional_data: &[u8]) -> Result<(), CryptoError> {
    todo!()
  }

  fn open<'a>(
    &self,
    _data: &'a mut [u8],
    _additional_data: &[u8],
  ) -> Result<&'a mut [u8], CryptoError> {
    todo!()
  }
}

impl HeaderKey for PassthroughKey {
  fn decrypt(&self, _pn_offset: usize, _packet: &mut [u8]) {}

  fn encrypt(&self, _pn_offset: usize, _packet: &mut [u8]) {}

  fn sample_size(&self) -> usize {
    32
  }
}

impl PacketKey for PassthroughKey {
  fn encrypt(&self, _packet: u64, _buf: &mut [u8], _header_len: usize) {}

  fn decrypt(
    &self,
    _packet: u64,
    _header: &[u8],
    _payload: &mut BytesMut,
  ) -> Result<(), CryptoError> {
    Ok(())
  }

  fn tag_len(&self) -> usize {
    0
  }

  fn confidentiality_limit(&self) -> u64 {
    u64::MAX
  }

  fn integrity_limit(&self) -> u64 {
    todo!();
  }
}

struct PassthroughServerConfig {
  id: Hash,
}

impl crypto::ServerConfig for PassthroughServerConfig {
  fn initial_keys(
    &self,
    _version: u32,
    _dst_cid: &ConnectionId,
  ) -> Result<Keys, UnsupportedVersion> {
    Ok(Keys {
      header: KeyPair {
        local: Box::new(PassthroughKey),
        remote: Box::new(PassthroughKey),
      },
      packet: KeyPair {
        local: Box::new(PassthroughKey),
        remote: Box::new(PassthroughKey),
      },
    })
  }

  fn retry_tag(&self, _version: u32, _orig_dst_cid: &ConnectionId, _packet: &[u8]) -> [u8; 16] {
    todo!()
    // Default::default()
  }

  fn start_session(
    self: Arc<Self>,
    _version: u32,
    params: &TransportParameters,
  ) -> Box<dyn Session> {
    Box::new(PassthroughSession::new(self.id, None, Side::Server, params))
  }
}

pub(crate) struct PassthroughSession {
  id: Hash,
  params: TransportParameters,
  remote_id: Option<Hash>,
  remote_params: Option<TransportParameters>,
  side: Side,
  state: State,
}

struct PassthroughClientConfig {
  id: Hash,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
  Initial,
  ZeroRtt,
  Handshake,
  OneRtt,
  Data,
}

impl crypto::ClientConfig for PassthroughClientConfig {
  fn start_session(
    self: Arc<Self>,
    _version: u32,
    server_name: &str,
    params: &TransportParameters,
  ) -> Result<Box<dyn Session>, ConnectError> {
    Ok(Box::new(PassthroughSession::new(
      self.id,
      Some(server_name.parse::<Hash>().unwrap()),
      Side::Client,
      params,
    )))
  }
}

impl PassthroughSession {
  fn new(id: Hash, remote_id: Option<Hash>, side: Side, params: &TransportParameters) -> Self {
    Self {
      side,
      id,
      remote_id,
      params: *params,
      remote_params: None,
      state: State::Initial,
    }
  }

  pub(crate) fn endpoint(id: Hash, address: IpAddr, port: u16) -> Endpoint {
    log::trace!("getting endpoint for passthrough session");

    let mut endpoint = Endpoint::server(
      ServerConfig::new(
        Arc::new(PassthroughServerConfig { id }),
        Arc::new(PassthroughKey),
      ),
      (address, port).into(),
    )
    .unwrap();

    endpoint.set_default_client_config(ClientConfig::new(Arc::new(PassthroughClientConfig { id })));

    endpoint
  }

  fn log(&self, message: &str) {
    log::debug!(
      "{}: {:?}: {message}",
      match self.side {
        Side::Server => "server",
        Side::Client => "client",
      },
      self.state,
    );
  }
}

impl Session for PassthroughSession {
  fn initial_keys(&self, _dst_cid: &ConnectionId, _side: Side) -> Keys {
    self.log("initial_keys");
    PassthroughKey::keys()
  }

  fn handshake_data(&self) -> Option<Box<dyn Any>> {
    todo!()
  }

  fn peer_identity(&self) -> Option<Box<dyn Any>> {
    if let Some(remote_id) = self.remote_id {
      Some(Box::new(remote_id))
    } else {
      None
    }
  }

  fn early_crypto(&self) -> Option<(Box<dyn HeaderKey>, Box<dyn PacketKey>)> {
    self.log("early crypto");
    Some((Box::new(PassthroughKey), Box::new(PassthroughKey)))
  }

  fn early_data_accepted(&self) -> Option<bool> {
    Some(true)
  }

  fn is_handshaking(&self) -> bool {
    self.log(&format!("is_handshaking"));
    self.state != State::Data
  }

  fn read_handshake(&mut self, buf: &[u8]) -> Result<bool, TransportError> {
    self.log(&format!("read handshake: {buf:x?}"));
    let array: [u8; Hash::LEN] = buf[..Hash::LEN].try_into().unwrap();
    let remote_id = Hash::from(array);
    if let Some(expected_id) = self.remote_id {
      assert_eq!(remote_id, expected_id);
    } else {
      self.remote_id = Some(remote_id);
    }
    self.remote_params =
      Some(TransportParameters::read(self.side, &mut Cursor::new(&buf[Hash::LEN..])).unwrap());
    match (self.state, self.side) {
      (State::Initial, Side::Server) => {
        self.state = State::ZeroRtt;
      }
      (State::Handshake, Side::Client) => {
        self.state = State::OneRtt;
      }
      _ => panic!(),
    }

    Ok(true)
  }

  fn transport_parameters(&self) -> Result<Option<TransportParameters>, TransportError> {
    self.log("transport_parameters");
    if self.state == State::Handshake && self.side == Side::Client {
      Ok(Some(self.params))
    } else {
      Ok(self.remote_params)
    }
  }

  fn write_handshake(&mut self, buf: &mut Vec<u8>) -> Option<Keys> {
    self.log("write_handshake");
    match (self.state, self.side) {
      (State::Initial, Side::Client) => {
        buf.extend_from_slice(self.id.as_bytes());
        self.params.write(buf);
        self.state = State::ZeroRtt;
        None
      }
      (State::ZeroRtt, _) => {
        self.state = State::Handshake;
        Some(PassthroughKey::keys())
      }
      (State::Handshake, Side::Server) => {
        buf.extend_from_slice(self.id.as_bytes());
        self.params.write(buf);
        self.state = State::Data;
        Some(PassthroughKey::keys())
      }
      (State::OneRtt, _) => {
        self.state = State::Data;
        Some(PassthroughKey::keys())
      }
      _ => None,
    }
  }

  fn next_1rtt_keys(&mut self) -> Option<KeyPair<Box<dyn PacketKey>>> {
    self.log("next_1rtt_keys");
    Some(KeyPair {
      local: Box::new(PassthroughKey),
      remote: Box::new(PassthroughKey),
    })
  }

  fn is_valid_retry(&self, _orig_dst_cid: &ConnectionId, _header: &[u8], _payload: &[u8]) -> bool {
    todo!()
  }

  fn export_keying_material(
    &self,
    _output: &mut [u8],
    _label: &[u8],
    _context: &[u8],
  ) -> Result<(), ExportKeyingMaterialError> {
    todo!()
  }
}
