use super::*;

// Number of buckets a in node's routing table. For each bucket with position
// `i` in the routing table, we store nodes at distance `i` from ourselves.
// Note that this include nodes who have the same ID as our own, which reside
// at distance 0,
const BUCKETS: usize = 257;

// Maximum size of each routing table bucket, as well as the maximum number of
// nodes returned from FIND_NODE and FIND_VALUE query.
const K: usize = 20;

pub(crate) struct Node {
  pub(crate) address: IpAddr,
  pub(crate) directory: RwLock<HashMap<Hash, HashSet<Contact>>>,
  pub(crate) id: Hash,
  pub(crate) port: u16,
  pub(crate) received: AtomicU64,
  pub(crate) routing_table: RwLock<Vec<Vec<Contact>>>,
  pub(crate) sent: AtomicU64,
  pub(crate) socket: UdpSocket,
}

impl Node {
  pub(crate) async fn new(address: IpAddr, port: u16) -> io::Result<Self> {
    let socket = UdpSocket::bind((address, port)).await?;
    let socket_address = socket.local_addr()?;
    let mut rng = rand::thread_rng();

    Ok(Self {
      address: socket_address.ip(),
      directory: RwLock::default(),
      id: Hash::from(std::array::from_fn(|_| rng.gen())),
      port: socket_address.port(),
      received: AtomicU64::default(),
      routing_table: RwLock::new((0..=BUCKETS).map(|_| Default::default()).collect()),
      sent: AtomicU64::default(),
      socket,
    })
  }

  pub(crate) async fn run(&self, bootstrap: Option<Contact>) -> io::Result<()> {
    if let Some(bootstrap) = bootstrap {
      self.ping(bootstrap).await?;
    }

    loop {
      if let Err(err) = self.receive().await {
        eprintln!("DHT node error: {err}");
      }
    }
  }

  pub(crate) async fn store(&self, hash: Hash) -> io::Result<()> {
    for contact in self.routes(hash).await {
      self.send(contact, Payload::Store(hash)).await?;
    }
    Ok(())
  }

  async fn ping(&self, contact: Contact) -> io::Result<()> {
    self.send(contact, Payload::Ping).await
  }

  #[cfg(test)]
  fn contact(&self) -> Contact {
    Contact {
      address: self.address,
      port: self.port,
      id: self.id,
    }
  }

  async fn send(&self, contact: Contact, payload: Payload) -> io::Result<()> {
    let message = Message {
      payload,
      from: self.id,
    }
    .to_cbor();

    assert!(message.len() <= MAX_UDP_PAYLOAD);

    let sent = self
      .socket
      .send_to(&message, SocketAddr::from((contact.address, contact.port)))
      .await?;

    assert_eq!(sent, message.len());

    self.sent.fetch_add(1, atomic::Ordering::Relaxed);

    Ok(())
  }

  async fn receive(&self) -> io::Result<()> {
    let mut buffer = [0; MAX_UDP_PAYLOAD];

    let (received, from) = self.socket.recv_from(&mut buffer).await?;

    let message = Message::from_cbor(&buffer[0..received]).unwrap();

    let contact = Contact {
      address: from.ip(),
      id: message.from,
      port: from.port(),
    };

    self.update(contact).await;

    self.received.fetch_add(1, atomic::Ordering::Relaxed);

    match message.payload {
      Payload::Ping => self.send(contact, Payload::Pong).await?,
      Payload::Pong => {}
      Payload::Store(hash) => {
        self
          .directory
          .write()
          .await
          .entry(hash)
          .or_default()
          .insert(contact);
      }
    }

    Ok(())
  }

  async fn routes(&self, id: Hash) -> Vec<Contact> {
    let i = Distance::new(self.id, id).bucket();

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

  async fn update(&self, contact: Contact) {
    let i = Distance::new(self.id, contact.id).bucket();

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
  async fn bootstrap() -> io::Result<()> {
    let loopback = Ipv4Addr::new(127, 0, 0, 1).into();

    let bootstrap = Node::new(loopback, 0).await?;

    let node = Node::new(loopback, 0).await?;

    node.ping(bootstrap.contact()).await?;

    bootstrap.receive().await.unwrap();

    assert_eq!(bootstrap.routes(node.id).await, &[node.contact()]);

    node.receive().await.unwrap();

    assert_eq!(node.routes(bootstrap.id).await, &[bootstrap.contact()]);

    Ok(())
  }
}
