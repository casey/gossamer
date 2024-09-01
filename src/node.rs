use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub(crate) enum Error {
  Accept {
    address: SocketAddr,
    backtrace: Option<Backtrace>,
    source: quinn::ConnectionError,
  },
  Connect {
    backtrace: Option<Backtrace>,
    peer: Peer,
    source: quinn::ConnectError,
  },
  Connection {
    backtrace: Option<Backtrace>,
    peer: Peer,
    source: quinn::ConnectionError,
  },
  Deserialize {
    backtrace: Option<Backtrace>,
    peer: Peer,
    source: ciborium::de::Error<io::Error>,
  },
  LocalAddress {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  Read {
    backtrace: Option<Backtrace>,
    peer: Peer,
    source: quinn::ReadExactError,
  },
  Stop {
    backtrace: Option<Backtrace>,
    peer: Peer,
    source: quinn::StoppedError,
  },
  Write {
    backtrace: Option<Backtrace>,
    peer: Peer,
    source: quinn::WriteError,
  },
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum Status {
  Done,
}

impl Status {
  fn error_code(self) -> quinn::VarInt {
    quinn::VarInt::from_u32(match self {
      Self::Done => 0,
    })
  }
}

type Result<T = (), E = Error> = std::result::Result<T, E>;

pub(crate) struct Node {
  endpoint: Endpoint,
  id: Id,
  ip: IpAddr,
  pub(crate) port: u16,
  pub(crate) received: AtomicU64,
  pub(crate) local: RwLock<HashMap<Id, Peer>>,
  pub(crate) sent: AtomicU64,
  pub(crate) packages: Arc<BTreeMap<Hash, Package>>,
}

fn random_id() -> Id {
  let mut rng = rand::thread_rng();
  std::array::from_fn(|_| rng.gen()).into()
}

impl Node {
  pub(crate) async fn new(
    address: IpAddr,
    packages: BTreeMap<Hash, Package>,
    port: u16,
  ) -> Result<Self> {
    let id = random_id();

    let endpoint = passthrough::Session::endpoint(id, address, port);

    let socket_address = endpoint.local_addr().context(LocalAddressError)?;

    Ok(Self {
      endpoint,
      id,
      ip: socket_address.ip(),
      packages: Arc::new(packages),
      port: socket_address.port(),
      received: AtomicU64::default(),
      sent: AtomicU64::default(),
      local: RwLock::default(),
    })
  }

  pub(crate) fn peer(&self) -> Peer {
    Peer {
      id: self.id,
      ip: self.ip,
      port: self.port,
    }
  }

  pub(crate) async fn run(self: Arc<Self>) -> Result {
    const MULTICAST_IP: Ipv4Addr = Ipv4Addr::new(239, 4, 9, 151);
    // generic multicast application port:
    // https://datatracker.ietf.org/doc/draft-karstens-pim-multicast-application-ports/
    const MULTICAST_PORT: u16 = 49151;
    const MULTICAST_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(MULTICAST_IP), MULTICAST_PORT);

    log::info!("local peer: {}", self.peer());

    #[derive(Deserialize, Serialize)]
    struct Advertisement {
      id: Id,
      port: u16,
    }

    let socket = {
      use socket2::{Domain, Protocol, Socket, Type};

      let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

      #[cfg(unix)]
      socket.set_reuse_port(true).unwrap();

      socket
        .bind(&SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), MULTICAST_PORT).into())
        .unwrap();

      let socket = std::net::UdpSocket::from(socket);

      let socket = tokio::net::UdpSocket::from_std(socket).unwrap();

      socket
        .join_multicast_v4(MULTICAST_IP, Ipv4Addr::UNSPECIFIED)
        .unwrap();

      socket
    };

    let tx = Arc::new(socket);
    let rx = tx.clone();
    let id = self.id;
    let port = self.port;

    let node = self.clone();
    tokio::spawn(async move {
      loop {
        let mut buf = [0u8; u16::MAX as usize];

        let (len, src) = rx.recv_from(&mut buf).await.unwrap();

        let advertisement = Advertisement::from_cbor(&buf[..len]).unwrap();

        let peer = Peer {
          id: advertisement.id,
          ip: src.ip(),
          port: advertisement.port,
        };

        if peer.id == id {
          eprintln!("ignoring advertisement from self");
        } else {
          eprintln!("advertisement from peer: {peer}");
          node.ping(peer).await.unwrap();
        }
      }
    });

    tokio::spawn(async move {
      let advertisement = Advertisement { id, port }.to_cbor();

      // make a local connection to ensure we are accepting incoming
      // connections before sending our first advertisement
      let endpoint = passthrough::Session::endpoint(random_id(), Ipv4Addr::LOCALHOST.into(), 0);

      let peer = Peer {
        id,
        port,
        ip: Ipv4Addr::LOCALHOST.into(),
      };

      endpoint
        .connect((Ipv4Addr::LOCALHOST, port).into(), &id.to_string())
        .context(ConnectError { peer })
        .unwrap()
        .await
        .context(ConnectionError { peer })
        .unwrap();

      loop {
        tx.send_to(&advertisement, &MULTICAST_ADDR).await.unwrap();

        tokio::time::sleep(Duration::from_secs(5 * 60)).await;
      }
    });

    log::info!("listening for incoming connections");
    while let Some(incoming) = self.endpoint.accept().await {
      log::info!("accepted incoming connection");
      let clone = self.clone();
      tokio::spawn(async move {
        if let Err(err) = clone.accept(incoming).await {
          err.report();
        }
      });
    }

    Ok(())
  }

  async fn accept(self: Arc<Self>, incoming: Incoming) -> Result {
    let address = incoming.remote_address();

    let connection = incoming
      .accept()
      .context(AcceptError { address })?
      .await
      .context(AcceptError { address })?;

    let socket_addr = connection.remote_address();

    let peer = Peer {
      ip: socket_addr.ip(),
      id: passthrough::Session::peer_identity(&connection),
      port: socket_addr.port(),
    };

    let (mut tx, rx) = connection
      .accept_bi()
      .await
      .context(AcceptError { address })?;

    let message = self.receive(peer, rx).await?;

    self.received.fetch_add(1, atomic::Ordering::Relaxed);

    match message {
      Message::Get(hash) => {
        self
          .send(
            peer,
            &mut tx,
            response::Get(
              self
                .packages
                .get(&hash)
                .map(|package| package.manifest.to_cbor()),
            ),
          )
          .await?;

        Self::finish(connection, peer, Status::Done, tx).await?;
      }
      Message::Ping => self.send(peer, &mut tx, response::Ping).await?,
      Message::Search => {
        self
          .send(
            peer,
            &mut tx,
            response::Search(self.packages.keys().cloned().collect()),
          )
          .await?;
        Self::finish(connection, peer, Status::Done, tx).await?;
      }
    }

    Ok(())
  }

  async fn send<T: Serialize>(&self, peer: Peer, stream: &mut SendStream, message: T) -> Result {
    let message = message.to_cbor();

    assert!(message.len() < u16::MAX as usize);

    let len = message.len() as u16;

    stream
      .write_all(&len.to_le_bytes())
      .await
      .context(WriteError { peer })?;

    stream
      .write_all(&message)
      .await
      .context(WriteError { peer })?;

    stream.stopped().await.context(StopError { peer })?;

    self.sent.fetch_add(1, atomic::Ordering::Relaxed);

    Ok(())
  }

  async fn receive<T: DeserializeOwned>(&self, peer: Peer, mut rx: RecvStream) -> Result<T> {
    let mut len = [0; 2];

    rx.read_exact(&mut len).await.context(ReadError { peer })?;

    let len = u16::from_le_bytes(len) as usize;

    let mut buffer = vec![0; len];

    rx.read_exact(&mut buffer)
      .await
      .context(ReadError { peer })?;

    T::from_cbor(&buffer).context(DeserializeError { peer })
  }

  pub(crate) async fn connect(&self, peer: Peer) -> Result<Connection> {
    let connection = self
      .endpoint
      .connect(peer.socket_addr(), &peer.id.to_string())
      .context(ConnectError { peer })?
      .await
      .context(ConnectionError { peer })?;

    assert_eq!(passthrough::Session::peer_identity(&connection), peer.id);

    Ok(connection)
  }

  pub(crate) async fn ping(&self, peer: Peer) -> Result {
    log::debug!("pinging {peer}");

    self.check(peer).await?;

    self.local.write().await.insert(peer.id, peer);

    Ok(())
  }

  async fn find(&self, id: Id) -> Option<Peer> {
    self.local.read().await.get(&id).copied()
  }

  async fn check(&self, peer: Peer) -> Result {
    let connection = self
      .endpoint
      .connect(peer.socket_addr(), &peer.id.to_string())
      .context(ConnectError { peer })?
      .await
      .context(ConnectionError { peer })?;

    assert_eq!(passthrough::Session::peer_identity(&connection), peer.id);

    let (mut tx, rx) = connection
      .open_bi()
      .await
      .context(ConnectionError { peer })?;

    self.send(peer, &mut tx, Message::Ping).await?;

    assert!(matches!(self.receive(peer, rx).await?, response::Ping));

    Self::finish(connection, peer, Status::Done, tx).await?;

    Ok(())
  }

  async fn finish(
    connection: Connection,
    peer: Peer,
    status: Status,
    mut tx: SendStream,
  ) -> Result {
    tx.stopped().await.context(StopError { peer })?;

    connection.close(status.error_code(), &status.to_cbor());

    Ok(())
  }

  pub(crate) async fn get(&self, id: Id, package: Hash) -> Result<Option<Manifest>> {
    let Some(peer) = self.find(id).await else {
      return Ok(None);
    };

    let connection = self
      .endpoint
      .connect(peer.socket_addr(), &peer.id.to_string())
      .context(ConnectError { peer })?
      .await
      .context(ConnectionError { peer })?;

    assert_eq!(passthrough::Session::peer_identity(&connection), peer.id);

    let (mut tx, rx) = connection
      .open_bi()
      .await
      .context(ConnectionError { peer })?;

    self.send(peer, &mut tx, Message::Get(package)).await?;

    let response::Get(file) = self.receive(peer, rx).await?;

    Self::finish(connection, peer, Status::Done, tx).await?;

    Ok(file.map(|file| Manifest::from_cbor(&file).unwrap()))
  }

  pub(crate) async fn search(&self, id: Id) -> Result<Option<Vec<Hash>>> {
    let Some((_id, &peer)) = self
      .local
      .read()
      .await
      .iter()
      .find(|(_id, peer)| peer.id == id)
    else {
      return Ok(None);
    };

    let connection = self.connect(peer).await?;

    let (mut tx, rx) = connection
      .open_bi()
      .await
      .context(ConnectionError { peer })?;

    self.send(peer, &mut tx, Message::Search).await?;

    let response::Search(results) = self.receive(peer, rx).await?;

    Self::finish(connection, peer, Status::Done, tx).await?;

    Ok(Some(results))
  }
}
