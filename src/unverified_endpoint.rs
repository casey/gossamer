use {
  super::*,
  bytes::BytesMut,
  quinn::{
    crypto::{
      self, rustls::QuicClientConfig, CryptoError, ExportKeyingMaterialError, HeaderKey, KeyPair,
      Keys, PacketKey, Session, UnsupportedVersion,
    },
    rustls::{
      self,
      client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
      crypto::{ring, verify_tls12_signature, verify_tls13_signature, CryptoProvider},
      pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName, UnixTime},
      DigitallySignedStruct, SignatureScheme,
    },
    ClientConfig, Endpoint, ServerConfig,
  },
  quinn_proto::{transport_parameters::TransportParameters, ConnectionId, Side, TransportError},
};

#[derive(Debug)]
pub(crate) struct UnverifiedEndpoint(Arc<CryptoProvider>);

struct PassthroughKey;

impl HeaderKey for PassthroughKey {
  fn decrypt(&self, _pn_offset: usize, _packet: &mut [u8]) {}
  fn encrypt(&self, _pn_offset: usize, _packet: &mut [u8]) {}
  fn sample_size(&self) -> usize {
    0
  }
}

impl PacketKey for PassthroughKey {
  // Required methods
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
    u64::MAX
  }
}

struct PassthroughServerConfig;

impl crypto::ServerConfig for PassthroughServerConfig {
  fn initial_keys(
    &self,
    _version: u32,
    _dst_cid: &ConnectionId,
  ) -> Result<Keys, UnsupportedVersion> {
    todo!()
    // Ok(Keys {
    //   header: KeyPair {
    //     local: Box::new(PassthroughKey),
    //     remote: Box::new(PassthroughKey),
    //   },
    //   packet: KeyPair {
    //     local: Box::new(PassthroughKey),
    //     remote: Box::new(PassthroughKey),
    //   },
    // })
  }

  fn retry_tag(&self, _version: u32, _orig_dst_cid: &ConnectionId, _packet: &[u8]) -> [u8; 16] {
    todo!()
    // Default::default()
  }

  fn start_session(
    self: Arc<Self>,
    _version: u32,
    _params: &TransportParameters,
  ) -> Box<dyn Session> {
    todo!()
    // Box::new(PassthroughSession)
  }
}

struct PassthroughSession;

impl Session for PassthroughSession {
  fn initial_keys(&self, _dst_cid: &ConnectionId, _side: Side) -> Keys {
    todo!()
    // Keys {
    //   header: KeyPair {
    //     local: Box::new(PassthroughKey),
    //     remote: Box::new(PassthroughKey),
    //   },
    //   packet: KeyPair {
    //     local: Box::new(PassthroughKey),
    //     remote: Box::new(PassthroughKey),
    //   },
    // }
  }

  fn handshake_data(&self) -> Option<Box<dyn Any>> {
    todo!()
  }

  fn peer_identity(&self) -> Option<Box<dyn Any>> {
    todo!()
  }

  fn early_crypto(&self) -> Option<(Box<dyn HeaderKey>, Box<dyn PacketKey>)> {
    todo!()
  }

  fn early_data_accepted(&self) -> Option<bool> {
    todo!()
  }

  fn is_handshaking(&self) -> bool {
    todo!()
  }

  fn read_handshake(&mut self, _buf: &[u8]) -> Result<bool, TransportError> {
    todo!()
  }

  fn transport_parameters(&self) -> Result<Option<TransportParameters>, TransportError> {
    todo!()
  }

  fn write_handshake(&mut self, _buf: &mut Vec<u8>) -> Option<Keys> {
    todo!()
  }

  fn next_1rtt_keys(&mut self) -> Option<KeyPair<Box<dyn PacketKey>>> {
    todo!()
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
