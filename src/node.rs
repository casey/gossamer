use super::*;

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
  pub(crate) directory: RwLock<HashMap<Hash, HashSet<Peer>>>,
  pub(crate) endpoint: Endpoint,
  pub(crate) id: Hash,
  pub(crate) ip: IpAddr,
  pub(crate) port: u16,
  pub(crate) received: AtomicU64,
  pub(crate) routing_table: RwLock<Vec<Vec<Peer>>>,
  pub(crate) sent: AtomicU64,
}

impl Node {
  pub(crate) async fn new(address: IpAddr, port: u16) -> Result<Self> {
    let mut rng = rand::thread_rng();

    let id = Hash::from(std::array::from_fn(|_| rng.gen()));

    let endpoint = passthrough::Session::endpoint(id, address, port);

    let socket_address = endpoint.local_addr().context(LocalAddressError)?;

    Ok(Self {
      directory: RwLock::default(),
      endpoint,
      id,
      ip: socket_address.ip(),
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

  pub(crate) async fn run(self: Arc<Self>, bootstrap: Option<Peer>) -> Result {
    if let Some(bootstrap) = bootstrap {
      if let Err(err) = self.ping(bootstrap).await {
        err.report();
      }
    }

    while let Some(incoming) = self.endpoint.accept().await {
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

    self.update(peer).await;

    self.received.fetch_add(1, atomic::Ordering::Relaxed);

    match message {
      Message::FindNode(hash) => {
        self
          .send(peer, &mut tx, Message::Nodes(self.routes(hash).await))
          .await?
      }
      Message::Pong => todo!(),
      Message::Store(hash) => {
        self
          .directory
          .write()
          .await
          .entry(hash)
          .or_default()
          .insert(peer);
      }
      Message::Ping => self.send(peer, &mut tx, Message::Pong).await?,
      Message::Nodes(_) => {
        todo!()
      }
    }

    Self::finish(connection, peer, Status::Done, tx).await?;

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

  // pub(crate) async fn store(&self, hash: Hash) -> io::Result<()> {
  //   for contact in self.routes(hash).await {
  //     self.send(contact, Payload::Store(hash)).await?;
  //   }
  //   Ok(())
  // }

  async fn ping(&self, peer: Peer) -> Result {
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

    self.update(peer).await;

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

    let routing_table = self.routing_table.read().await;

    let mut peers = iter::once(&routing_table[i])
      .chain(routing_table[..i].iter().rev())
      .chain(&routing_table[i + 1..])
      .flat_map(|bucket| bucket.iter())
      .take(K)
      .copied()
      .collect::<Vec<Peer>>();

    peers.sort_by_key(|peer| Distance::new(id, peer.id));

    peers
  }

  async fn update(&self, peer: Peer) {
    let i = Distance::new(self.id, peer.id).bucket();

    let bucket = &mut self.routing_table.write().await[i];

    if let Some(i) = bucket.iter().copied().position(|c| c == peer) {
      bucket.remove(i);
      bucket.push(peer);
    } else if bucket.len() < K {
      bucket.push(peer);
    } else {
      eprintln!("routing table bucket {i} full, dropping contact")
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, std::net::Ipv4Addr};

  #[tokio::test]
  async fn bootstrap() -> Result {
    env_logger::init();

    let loopback = Ipv4Addr::new(127, 0, 0, 1).into();

    let bootstrap = Arc::new(Node::new(loopback, 0).await?);

    tokio::spawn(bootstrap.clone().run(None));

    let node = Node::new(loopback, 0).await?;

    node.ping(bootstrap.peer()).await?;

    assert_eq!(bootstrap.routes(node.id).await, &[node.peer()]);

    assert_eq!(node.routes(bootstrap.id).await, &[bootstrap.peer()]);

    Ok(())
  }
}
