use super::*;

// # Kademlia RPCs:
//
// ## PING
//
// sends an empty message to a node and updates its routing table if the node responds
//
// ## STORE
//
// store a value under a key in a node (usually the senders contact information under a hash)
//
// ## FIND_NODE
//
// get the K closest nodes to a hash
//
// ## FIND_VALUE
//
// get the K closed nodes to a hash, unless the node has received a STORE for that hash,
// in which case it returns the contact infornmation for that hah

// Number of buckets in a node's routing table. For each bucket with position
// `i` in the routing table, we store nodes at distance `i` from ourselves.
// Note that this include nodes who have the same ID as our own, which reside
// at distance 0,
const BUCKETS: usize = 257;

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
  UnexpectedMessage(&'static str),
}

impl Status {
  fn error_code(self) -> quinn::VarInt {
    quinn::VarInt::from_u32(match self {
      Self::Done => 0,
      Self::UnexpectedMessage(_) => 1,
    })
  }
}

type Result<T = (), E = Error> = std::result::Result<T, E>;

pub(crate) struct Node {
  pub(crate) directory: RwLock<HashMap<Hash, HashSet<Peer>>>,
  pub(crate) endpoint: Endpoint,
  pub(crate) id: Hash,
  pub(crate) ip: IpAddr,
  pub(crate) port: u16,
  pub(crate) received: AtomicU64,
  pub(crate) routing_table: RwLock<Vec<Vec<Peer>>>,
  pub(crate) sent: AtomicU64,
  library: Arc<Library>,
}

fn hash() -> Hash {
  let mut rng = rand::thread_rng();
  std::array::from_fn(|_| rng.gen()).into()
}

impl Node {
  pub(crate) async fn new(address: IpAddr, library: Arc<Library>, port: u16) -> Result<Self> {
    let id = hash();

    let endpoint = passthrough::Session::endpoint(id, address, port);

    let socket_address = endpoint.local_addr().context(LocalAddressError)?;

    Ok(Self {
      directory: RwLock::default(),
      endpoint,
      id,
      ip: socket_address.ip(),
      library,
      port: socket_address.port(),
      received: AtomicU64::default(),
      routing_table: RwLock::new((0..=BUCKETS).map(|_| Default::default()).collect()),
      sent: AtomicU64::default(),
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
      id: Hash,
      port: u16,
    }

    let socket = {
      use socket2::{Domain, Protocol, Socket, Type};

      let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

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
          // todo:
          // - don't ping if already in routing table
          // -
        }
      }
    });

    tokio::spawn(async move {
      let advertisement = Advertisement { id, port }.to_cbor();

      // make a local connection to ensure we are accepting incoming
      // connections before sending our first advertisement
      let endpoint = passthrough::Session::endpoint(hash(), Ipv4Addr::LOCALHOST.into(), 0);

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

    self.update(peer).await?;

    self.received.fetch_add(1, atomic::Ordering::Relaxed);

    match message {
      Message::FindNode(hash) => {
        self
          .send(peer, &mut tx, Message::Nodes(self.routes(hash).await))
          .await?;
        Self::finish(connection, peer, Status::Done, tx).await?;
      }
      Message::Get(hash) => {
        self
          .send(
            peer,
            &mut tx,
            Message::File(
              self
                .library
                .packages
                .get(&hash)
                .map(|package| package.manifest.to_cbor()),
            ),
          )
          .await?;

        Self::finish(connection, peer, Status::Done, tx).await?;
      }
      Message::Search => {
        self
          .send(
            peer,
            &mut tx,
            Message::Results(self.library.packages.keys().cloned().collect()),
          )
          .await?;
        Self::finish(connection, peer, Status::Done, tx).await?;
      }
      Message::Store(hash) => {
        self
          .directory
          .write()
          .await
          .entry(hash)
          .or_default()
          .insert(peer);
        Self::finish(connection, peer, Status::Done, tx).await?;
      }
      Message::Ping => self.send(peer, &mut tx, Message::Pong).await?,
      message @ (Message::Nodes(_) | Message::Pong | Message::Results(_) | Message::File(_)) => {
        Self::finish(
          connection,
          peer,
          Status::UnexpectedMessage(message.into()),
          tx,
        )
        .await?
      }
    }

    Ok(())
  }

  async fn send(&self, peer: Peer, stream: &mut SendStream, message: Message) -> Result {
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

  async fn receive(&self, peer: Peer, mut rx: RecvStream) -> Result<Message> {
    let mut len = [0; 2];

    rx.read_exact(&mut len).await.context(ReadError { peer })?;

    let len = u16::from_le_bytes(len) as usize;

    let mut buffer = vec![0; len];

    rx.read_exact(&mut buffer)
      .await
      .context(ReadError { peer })?;

    Message::from_cbor(&buffer).context(DeserializeError { peer })
  }

  pub(crate) async fn connect(&self, peer: Peer) -> Result<Connection> {
    let connection = self
      .endpoint
      .connect(peer.socket_addr(), &peer.id.to_string())
      .context(ConnectError { peer })?
      .await
      .context(ConnectionError { peer })?;

    assert_eq!(passthrough::Session::peer_identity(&connection), peer.id);

    self.update(peer).await?;

    Ok(connection)
  }

  pub(crate) async fn store(&self, hash: Hash) -> Result {
    for peer in self.routes(hash).await {
      let connection = self.connect(peer).await?;

      let (mut tx, _rx) = connection
        .open_bi()
        .await
        .context(ConnectionError { peer })?;

      self.send(peer, &mut tx, Message::Store(hash)).await?;

      Self::finish(connection, peer, Status::Done, tx).await?;
    }
    Ok(())
  }

  pub(crate) async fn ping(&self, peer: Peer) -> Result {
    log::debug!("pinging {peer}");

    self.check(peer).await?;
    self.update(peer).await?;

    Ok(())
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

    assert!(matches!(self.receive(peer, rx).await?, Message::Pong));

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

  async fn routes(&self, id: Hash) -> Vec<Peer> {
    let i = Distance::new(self.id, id).bucket();

    let mut heap = BinaryHeap::<(Distance, Peer)>::new();

    let routing_table = self.routing_table.read().await;

    for bucket in iter::once(&routing_table[i])
      .chain(routing_table[..i].iter().rev())
      .chain(&routing_table[i + 1..])
    {
      for peer in bucket {
        heap.push((Distance::new(id, peer.id), *peer));

        if heap.len() > K {
          heap.pop();
        }
      }
    }

    heap
      .into_sorted_vec()
      .into_iter()
      .map(|(_distance, peer)| peer)
      .collect()
  }

  pub(crate) async fn get(&self, id: Hash, package: Hash) -> Result<Option<Manifest>> {
    let bucket = Distance::new(self.id, id).bucket();

    let routing_table = self.routing_table.read().await;

    for &peer in &routing_table[bucket] {
      if peer.id == id {
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

        let message = self.receive(peer, rx).await?;

        let Message::File(file) = message else {
          todo!();
        };

        Self::finish(connection, peer, Status::Done, tx).await?;

        return Ok(file.map(|file| Manifest::from_cbor(&file).unwrap()));
      }
    }

    Ok(None)
  }

  pub(crate) async fn search(&self, id: Hash) -> Result<Option<BTreeSet<Hash>>> {
    if let Some(peer) = self.find(id).await {
      let connection = self.connect(peer).await?;

      let (mut tx, rx) = connection
        .open_bi()
        .await
        .context(ConnectionError { peer })?;

      self.send(peer, &mut tx, Message::Search).await?;

      let message = self.receive(peer, rx).await?;

      let Message::Results(results) = message else {
        todo!();
      };

      Self::finish(connection, peer, Status::Done, tx).await?;

      Ok(Some(results))
    } else {
      Ok(None)
    }
  }

  pub(crate) async fn find(&self, id: Hash) -> Option<Peer> {
    let routing_table = self.routing_table.read().await;

    for bucket in routing_table.iter() {
      for peer in bucket {
        if peer.id == id {
          return Some(*peer);
        }
      }
    }

    None
  }

  pub(crate) async fn update(&self, peer: Peer) -> Result {
    let i = Distance::new(self.id, peer.id).bucket();
    let bucket = &mut self.routing_table.write().await[i];

    if let Some(i) = bucket.iter().copied().position(|c| c == peer) {
      bucket.remove(i);
      bucket.push(peer);
    } else if bucket.len() < K {
      bucket.push(peer);
    } else {
      let oldest = bucket.remove(0);
      match self.check(oldest).await {
        Ok(()) => bucket.push(oldest),
        Err(err) => {
          log::trace!("peer {oldest} did not respond: {err}");
          bucket.push(peer);
        }
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, std::net::Ipv4Addr};

  #[tokio::test]
  async fn ping() -> Result {
    let loopback = Ipv4Addr::new(127, 0, 0, 1).into();

    let a = Arc::new(Node::new(loopback, 0).await?);

    tokio::spawn(a.clone().run());

    let b = Node::new(loopback, 0).await?;

    b.ping(a.peer()).await?;

    assert_eq!(a.routes(b.id).await, &[b.peer()]);

    assert_eq!(b.routes(a.id).await, &[a.peer()]);

    Ok(())
  }

  #[tokio::test]
  async fn store() -> Result {
    let loopback = Ipv4Addr::new(127, 0, 0, 1).into();

    let a = Arc::new(Node::new(loopback, 0).await?);

    tokio::spawn(a.clone().run());

    let b = Node::new(loopback, 0).await?;

    b.update(a.peer()).await?;

    b.store(a.id).await?;

    assert_eq!(a.routes(b.id).await, &[b.peer()]);

    assert_eq!(b.routes(a.id).await, &[a.peer()]);

    assert!(a.directory.read().await[&a.id].contains(&b.peer()));

    Ok(())
  }
}
