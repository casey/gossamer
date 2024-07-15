use {
  super::*,
  bytes::BytesMut,
  quinn::{
    crypto::{
      self, AeadKey, CryptoError, ExportKeyingMaterialError, HandshakeTokenKey, HeaderKey, KeyPair,
      Keys, PacketKey, UnsupportedVersion,
    },
    ConnectError, Endpoint,
  },
  quinn_proto::{transport_parameters::TransportParameters, ConnectionId, Side, TransportError},
};

struct Key;

impl Key {
  fn keys() -> Keys {
    Keys {
      header: KeyPair {
        local: Box::new(Key),
        remote: Box::new(Key),
      },
      packet: KeyPair {
        local: Box::new(Key),
        remote: Box::new(Key),
      },
    }
  }
}

impl AeadKey for Key {
  fn open<'a>(
    &self,
    _data: &'a mut [u8],
    _additional_data: &[u8],
  ) -> Result<&'a mut [u8], CryptoError> {
    todo!()
  }

  fn seal(&self, _data: &mut Vec<u8>, _additional_data: &[u8]) -> Result<(), CryptoError> {
    todo!()
  }
}

impl HandshakeTokenKey for Key {
  fn aead_from_hkdf(&self, _random_bytes: &[u8]) -> Box<dyn AeadKey> {
    todo!()
  }
}

impl HeaderKey for Key {
  fn decrypt(&self, _pn_offset: usize, _packet: &mut [u8]) {}

  fn encrypt(&self, _pn_offset: usize, _packet: &mut [u8]) {}

  fn sample_size(&self) -> usize {
    32
  }
}

impl PacketKey for Key {
  fn confidentiality_limit(&self) -> u64 {
    u64::MAX
  }

  fn decrypt(
    &self,
    _packet: u64,
    _header: &[u8],
    _payload: &mut BytesMut,
  ) -> Result<(), CryptoError> {
    Ok(())
  }

  fn encrypt(&self, _packet: u64, _buf: &mut [u8], _header_len: usize) {}

  fn integrity_limit(&self) -> u64 {
    todo!();
  }

  fn tag_len(&self) -> usize {
    0
  }
}

struct ClientConfig {
  id: Id,
}

impl crypto::ClientConfig for ClientConfig {
  fn start_session(
    self: Arc<Self>,
    _version: u32,
    server_name: &str,
    params: &TransportParameters,
  ) -> Result<Box<dyn crypto::Session>, ConnectError> {
    Ok(Box::new(Session::new(
      self.id,
      Some(server_name.parse::<Id>().unwrap()),
      Side::Client,
      params,
    )))
  }
}

struct ServerConfig {
  id: Id,
}

impl crypto::ServerConfig for ServerConfig {
  fn initial_keys(
    &self,
    _version: u32,
    _dst_cid: &ConnectionId,
  ) -> Result<Keys, UnsupportedVersion> {
    Ok(Keys {
      header: KeyPair {
        local: Box::new(Key),
        remote: Box::new(Key),
      },
      packet: KeyPair {
        local: Box::new(Key),
        remote: Box::new(Key),
      },
    })
  }

  fn retry_tag(&self, _version: u32, _orig_dst_cid: &ConnectionId, _packet: &[u8]) -> [u8; 16] {
    todo!()
  }

  fn start_session(
    self: Arc<Self>,
    _version: u32,
    params: &TransportParameters,
  ) -> Box<dyn crypto::Session> {
    Box::new(Session::new(self.id, None, Side::Server, params))
  }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
  Initial,
  ZeroRtt,
  Handshake,
  OneRtt,
  Data,
}

pub(crate) struct Session {
  id: Id,
  params: TransportParameters,
  remote_id: Option<Id>,
  remote_params: Option<TransportParameters>,
  side: Side,
  state: State,
}

impl Session {
  fn new(id: Id, remote_id: Option<Id>, side: Side, params: &TransportParameters) -> Self {
    Self {
      id,
      params: *params,
      remote_id,
      remote_params: None,
      side,
      state: State::Initial,
    }
  }

  pub(crate) fn endpoint(id: Id, address: IpAddr, port: u16) -> Endpoint {
    let mut endpoint = Endpoint::server(
      quinn::ServerConfig::new(Arc::new(ServerConfig { id }), Arc::new(Key)),
      (address, port).into(),
    )
    .unwrap();

    endpoint.set_default_client_config(quinn::ClientConfig::new(Arc::new(ClientConfig { id })));

    endpoint
  }

  pub(crate) fn peer_identity(connection: &Connection) -> Id {
    *connection.peer_identity().unwrap().downcast().unwrap()
  }
}

impl crypto::Session for Session {
  fn early_crypto(&self) -> Option<(Box<dyn HeaderKey>, Box<dyn PacketKey>)> {
    Some((Box::new(Key), Box::new(Key)))
  }

  fn early_data_accepted(&self) -> Option<bool> {
    Some(true)
  }

  fn export_keying_material(
    &self,
    _output: &mut [u8],
    _label: &[u8],
    _context: &[u8],
  ) -> Result<(), ExportKeyingMaterialError> {
    todo!()
  }

  fn handshake_data(&self) -> Option<Box<dyn Any>> {
    todo!()
  }

  fn initial_keys(&self, _dst_cid: &ConnectionId, _side: Side) -> Keys {
    Key::keys()
  }

  fn is_handshaking(&self) -> bool {
    self.state != State::Data
  }

  fn is_valid_retry(&self, _orig_dst_cid: &ConnectionId, _header: &[u8], _payload: &[u8]) -> bool {
    todo!()
  }

  fn next_1rtt_keys(&mut self) -> Option<KeyPair<Box<dyn PacketKey>>> {
    Some(KeyPair {
      local: Box::new(Key),
      remote: Box::new(Key),
    })
  }

  fn peer_identity(&self) -> Option<Box<dyn Any>> {
    if let Some(remote_id) = self.remote_id {
      Some(Box::new(remote_id))
    } else {
      None
    }
  }

  fn read_handshake(&mut self, buf: &[u8]) -> Result<bool, TransportError> {
    let array: [u8; Id::LEN] = buf[..Id::LEN].try_into().unwrap();
    let remote_id = Id::from(array);
    if let Some(expected_id) = self.remote_id {
      assert_eq!(remote_id, expected_id);
    } else {
      self.remote_id = Some(remote_id);
    }
    self.remote_params =
      Some(TransportParameters::read(self.side, &mut Cursor::new(&buf[Id::LEN..])).unwrap());
    match (self.state, self.side) {
      (State::Initial, Side::Server) => {
        self.state = State::ZeroRtt;
      }
      (State::Handshake, Side::Client) => {
        self.state = State::OneRtt;
      }
      _ => todo!(),
    }

    Ok(true)
  }

  fn transport_parameters(&self) -> Result<Option<TransportParameters>, TransportError> {
    if self.state == State::Handshake && self.side == Side::Client {
      Ok(Some(self.params))
    } else {
      Ok(self.remote_params)
    }
  }

  fn write_handshake(&mut self, buf: &mut Vec<u8>) -> Option<Keys> {
    match (self.state, self.side) {
      (State::Initial, Side::Client) => {
        buf.extend_from_slice(self.id.as_bytes());
        self.params.write(buf);
        self.state = State::ZeroRtt;
        None
      }
      (State::ZeroRtt, _) => {
        self.state = State::Handshake;
        Some(Key::keys())
      }
      (State::Handshake, Side::Server) => {
        buf.extend_from_slice(self.id.as_bytes());
        self.params.write(buf);
        self.state = State::Data;
        Some(Key::keys())
      }
      (State::OneRtt, _) => {
        self.state = State::Data;
        Some(Key::keys())
      }
      _ => None,
    }
  }
}
