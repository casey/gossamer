use {
  super::*,
  quinn::{Connection, VarInt},
};

// Number of buckets in a node's routing table. For each bucket with position
// `i` in the routing table, we store nodes at distance `i` from ourselves.
// Note that this include nodes who have the same ID as our own, which reside
// at distance 0,
const BUCKETS: usize = 257;

pub(crate) struct Node {
  pub(crate) contact: Contact,
  pub(crate) directory: RwLock<HashMap<Hash, HashSet<Contact>>>,
  pub(crate) endpoint: Endpoint,
  pub(crate) received: AtomicU64,
  pub(crate) routing_table: RwLock<Vec<Vec<Contact>>>,
  pub(crate) sent: AtomicU64,
}

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub(crate) enum Error {
  Accept {
    backtrace: Option<Backtrace>,
    source: quinn::ConnectionError,
  },
  Connect {
    backtrace: Option<Backtrace>,
    source: quinn::ConnectError,
  },
  Connection {
    backtrace: Option<Backtrace>,
    source: quinn::ConnectionError,
  },
  DeserializeError {
    backtrace: Option<Backtrace>,
    source: ciborium::de::Error<io::Error>,
  },
  LocalAddress {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  Read {
    backtrace: Option<Backtrace>,
    source: quinn::ReadExactError,
  },
  Write {
    backtrace: Option<Backtrace>,
    source: quinn::WriteError,
  },
}

type Result<T = (), E = Error> = std::result::Result<T, E>;

impl Node {
  pub(crate) async fn new(address: IpAddr, port: u16) -> Result<Self> {
    let mut rng = rand::thread_rng();

    let id = Hash::from(std::array::from_fn(|_| rng.gen()));

    let endpoint = Session::endpoint(id, address, port);

    let socket_address = endpoint.local_addr().context(LocalAddressError)?;

    Ok(Self {
      contact: Contact {
        address: socket_address.ip(),
        port: socket_address.port(),
        id,
      },
      directory: RwLock::default(),
      endpoint,
      received: AtomicU64::default(),
      routing_table: RwLock::new((0..=BUCKETS).map(|_| Default::default()).collect()),
      sent: AtomicU64::default(),
    })
  }

  pub(crate) async fn run(self: Arc<Self>, bootstrap: Option<Contact>) -> Result {
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
    let connection = incoming
      .accept()
      .context(AcceptError)?
      .await
      .context(AcceptError)?;

    let from = connection.remote_address();

    let (mut tx, rx) = connection.accept_bi().await.context(AcceptError)?;

    let message = self.receive(rx).await?;

    let from = Contact {
      address: from.ip(),
      id: *connection.peer_identity().unwrap().downcast().unwrap(),
      port: from.port(),
    };

    self.update(from).await;

    self.received.fetch_add(1, atomic::Ordering::Relaxed);

    match message {
      Message::FindNode(hash) => {
        self
          .send(&mut tx, Message::Nodes(self.routes(hash).await))
          .await?
      }

      Message::Nodes(_) => {
        todo!()
      }
      Message::Ping => self.send(&mut tx, Message::Pong).await?,
      Message::Pong => todo!(),
      Message::Store(hash) => {
        self
          .directory
          .write()
          .await
          .entry(hash)
          .or_default()
          .insert(from);
      }
    }

    Self::finish(connection, tx).await;

    Ok(())
  }

  async fn send(&self, stream: &mut SendStream, message: Message) -> Result {
    let message = message.to_cbor();

    assert!(message.len() < u16::MAX as usize);

    let len = message.len() as u16;

    stream
      .write_all(&len.to_le_bytes())
      .await
      .context(WriteError)?;

    stream.write_all(&message).await.context(WriteError)?;

    stream.stopped().await.unwrap();

    self.sent.fetch_add(1, atomic::Ordering::Relaxed);

    Ok(())
  }

  async fn receive(&self, mut rx: RecvStream) -> Result<Message> {
    let mut len = [0; 2];

    rx.read_exact(&mut len).await.context(ReadError)?;

    let len = u16::from_le_bytes(len) as usize;

    let mut buffer = vec![0; len];

    rx.read_exact(&mut buffer).await.context(ReadError)?;

    Message::from_cbor(&buffer).context(DeserializeError)
  }

  // pub(crate) async fn store(&self, hash: Hash) -> io::Result<()> {
  //   for contact in self.routes(hash).await {
  //     self.send(contact, Payload::Store(hash)).await?;
  //   }
  //   Ok(())
  // }

  async fn ping(&self, contact: Contact) -> Result {
    let connection = self
      .endpoint
      .connect(
        (contact.address, contact.port).into(),
        &contact.id.to_string(),
      )
      .context(ConnectError)?
      .await
      .context(ConnectionError)?;

    assert_eq!(
      *connection
        .peer_identity()
        .unwrap()
        .downcast::<Hash>()
        .unwrap(),
      contact.id,
    );

    let (mut tx, rx) = connection.open_bi().await.context(ConnectionError)?;

    self.send(&mut tx, Message::Ping).await?;

    assert!(matches!(self.receive(rx).await?, Message::Pong));

    self.update(contact).await;

    Self::finish(connection, tx).await;

    Ok(())
  }

  async fn finish(connection: Connection, mut tx: SendStream) {
    tx.stopped().await.unwrap();

    connection.close(VarInt::from_u32(0), b"done");
  }

  async fn routes(&self, id: Hash) -> Vec<Contact> {
    let i = Distance::new(self.id(), id).bucket();

    let routing_table = self.routing_table.read().await;

    let mut contacts = iter::once(&routing_table[i])
      .chain(routing_table[..i].iter().rev())
      .chain(&routing_table[i + 1..])
      .flat_map(|bucket| bucket.iter())
      .take(K)
      .copied()
      .collect::<Vec<Contact>>();

    contacts.sort_by_key(|contact| Distance::new(id, contact.id));

    contacts
  }

  fn id(&self) -> Hash {
    self.contact.id
  }

  async fn update(&self, contact: Contact) {
    let i = Distance::new(self.id(), contact.id).bucket();

    let bucket = &mut self.routing_table.write().await[i];

    if let Some(i) = bucket.iter().copied().position(|c| c == contact) {
      bucket.remove(i);
      bucket.push(contact);
    } else if bucket.len() < K {
      bucket.push(contact);
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

    node.ping(bootstrap.contact).await?;

    assert_eq!(bootstrap.routes(node.id()).await, &[node.contact]);

    assert_eq!(node.routes(bootstrap.id()).await, &[bootstrap.contact]);

    Ok(())
  }
}
