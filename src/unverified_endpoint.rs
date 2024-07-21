use {
  super::*,
  bytes::BytesMut,
  quinn::{
    crypto::{
      self, rustls::QuicClientConfig, AeadKey, CryptoError, ExportKeyingMaterialError,
      HandshakeTokenKey, HeaderKey, KeyPair, Keys, PacketKey, Session, UnsupportedVersion,
    },
    rustls::{
      self,
      client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
      crypto::{ring, verify_tls12_signature, verify_tls13_signature, CryptoProvider},
      pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName, UnixTime},
      DigitallySignedStruct, SignatureScheme,
    },
    ClientConfig, ConnectError, Endpoint, ServerConfig,
  },
  quinn_proto::{transport_parameters::TransportParameters, ConnectionId, Side, TransportError},
};

#[derive(Debug)]
pub(crate) struct UnverifiedEndpoint(Arc<CryptoProvider>);

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
    data: &'a mut [u8],
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

struct PassthroughServerConfig;

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
    Box::new(PassthroughSession {
      side: Side::Server,
      n: 0,
      k: false,
      w: false,
      params: params.clone(),
      p: None,
    })
  }
}

pub(crate) struct PassthroughSession {
  side: Side,
  n: u32,
  params: TransportParameters,
  p: Option<TransportParameters>,
  k: bool,
  w: bool,
}

struct PassthroughClientConfig;

impl crypto::ClientConfig for PassthroughClientConfig {
  fn start_session(
    self: Arc<Self>,
    version: u32,
    server_name: &str,
    params: &TransportParameters,
  ) -> Result<Box<dyn Session>, ConnectError> {
    Ok(Box::new(PassthroughSession {
      n: 0,
      side: Side::Client,
      params: params.clone(),
      p: None,
      k: false,
      w: false,
    }))
  }
}

impl PassthroughSession {
  pub(crate) fn endpoint(address: IpAddr, port: u16) -> Endpoint {
    log::trace!("getting endpoint for passthrough session");

    let mut endpoint = Endpoint::server(
      ServerConfig::new(Arc::new(PassthroughServerConfig), Arc::new(PassthroughKey)),
      (address, port).into(),
    )
    .unwrap();

    endpoint.set_default_client_config(ClientConfig::new(Arc::new(PassthroughClientConfig)));

    endpoint
  }

  fn log(&self, message: &str) {
    log::debug!(
      "{}: {message}",
      match self.side {
        Side::Server => "server",
        Side::Client => "client",
      },
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
    todo!()
  }

  fn early_crypto(&self) -> Option<(Box<dyn HeaderKey>, Box<dyn PacketKey>)> {
    self.log("early crypto");
    None
  }

  fn early_data_accepted(&self) -> Option<bool> {
    todo!()
  }

  fn is_handshaking(&self) -> bool {
    self.log("is_handshaking");
    !self.k || !self.w || self.p.is_none()
  }

  fn read_handshake(&mut self, buf: &[u8]) -> Result<bool, TransportError> {
    self.log("read handshake");
    let result = self.p.is_none();
    self.p = Some(TransportParameters::read(self.side, &mut Cursor::new(buf)).unwrap());
    Ok(result)
  }

  fn transport_parameters(&self) -> Result<Option<TransportParameters>, TransportError> {
    self.log("transport_parameters");
    Ok(self.p.clone())
  }

  fn write_handshake(&mut self, buf: &mut Vec<u8>) -> Option<Keys> {
    self.log("write_handshake");

    if !self.w {
      self.w = true;
      self.params.write(buf);
    }

    if self.p.is_some() && !self.k {
      self.k = true;
      return Some(PassthroughKey::keys());
    }

    None
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

impl UnverifiedEndpoint {
  pub(crate) const SERVER_NAME: &'static str = "localhost";

  pub(crate) fn new(address: IpAddr, port: u16) -> Endpoint {
    let cert = rcgen::generate_simple_self_signed(vec![Self::SERVER_NAME.into()]).unwrap();
    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());

    let mut server_config =
      ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into()).unwrap();
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let mut endpoint = Endpoint::server(server_config, (address, port).into()).unwrap();

    endpoint.set_default_client_config(ClientConfig::new(Arc::new(
      QuicClientConfig::try_from(
        rustls::ClientConfig::builder()
          .dangerous()
          .with_custom_certificate_verifier(Arc::new(Self(Arc::new(ring::default_provider()))))
          .with_no_client_auth(),
      )
      .unwrap(),
    )));

    endpoint
  }
}

impl ServerCertVerifier for UnverifiedEndpoint {
  fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
    self.0.signature_verification_algorithms.supported_schemes()
  }

  fn verify_server_cert(
    &self,
    _end_entity: &CertificateDer<'_>,
    _intermediates: &[CertificateDer<'_>],
    _server_name: &ServerName<'_>,
    _ocsp: &[u8],
    _now: UnixTime,
  ) -> Result<ServerCertVerified, rustls::Error> {
    Ok(ServerCertVerified::assertion())
  }

  fn verify_tls12_signature(
    &self,
    message: &[u8],
    cert: &CertificateDer<'_>,
    dss: &DigitallySignedStruct,
  ) -> Result<HandshakeSignatureValid, rustls::Error> {
    verify_tls12_signature(
      message,
      cert,
      dss,
      &self.0.signature_verification_algorithms,
    )
  }

  fn verify_tls13_signature(
    &self,
    message: &[u8],
    cert: &CertificateDer<'_>,
    dss: &DigitallySignedStruct,
  ) -> Result<HandshakeSignatureValid, rustls::Error> {
    verify_tls13_signature(
      message,
      cert,
      dss,
      &self.0.signature_verification_algorithms,
    )
  }
}
