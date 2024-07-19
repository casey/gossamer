use {
  super::*,
  quinn::{
    crypto::rustls::QuicClientConfig,
    rustls::{
      self,
      client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
      crypto::{ring, verify_tls12_signature, verify_tls13_signature, CryptoProvider},
      pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName, UnixTime},
      DigitallySignedStruct, SignatureScheme,
    },
    ClientConfig, Endpoint, ServerConfig,
  },
};

#[derive(Debug)]
pub(crate) struct UnverifiedEndpoint(Arc<CryptoProvider>);

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
