use {
  self::{
    server_error::ServerError,
    templates::{NodeHtml, PackageHtml, PageHtml, SearchHtml},
  },
  super::*,
  axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
  },
  rust_embed::RustEmbed,
  tokio::runtime::Runtime,
};

// todo:
// - okay_or_not_found

mod server_error;
mod templates;

#[derive(RustEmbed)]
#[folder = "static"]
struct StaticAssets;

#[derive(Parser)]
pub(crate) struct Server {
  #[arg(
    long,
    help = "Listen on <ADDRESS> for incoming requests.",
    default_value = "::"
  )]
  address: IpAddr,
  #[arg(
    long,
    help = "Listen on <PORT> for incoming HTTP requests.",
    default_value = "80"
  )]
  http_port: u16,
  #[arg(long, help = "Load <PACKAGE> into library.", value_name = "<PACKAGE>", num_args = 0..)]
  packages: Vec<Utf8PathBuf>,
  #[arg(long, help = "Open server in browser.")]
  open: bool,
  #[arg(long, help = "Bootstrap DHT node with <PEER>.", value_name = "<PEER>")]
  bootstrap: Option<Peer>,
}

#[derive(Debug)]
struct Resource {
  content_type: Mime,
  content: Vec<u8>,
}

impl Resource {
  fn new(content_type: Mime, content: Vec<u8>) -> Self {
    Self {
      content_type,
      content,
    }
  }
}

impl IntoResponse for Resource {
  fn into_response(self) -> Response<Body> {
    (
      [(header::CONTENT_TYPE, self.content_type.to_string())],
      self.content,
    )
      .into_response()
  }
}

type ServerResult<T = Resource> = std::result::Result<T, ServerError>;

impl Server {
  pub(crate) fn run(self) -> Result {
    let mut packages = BTreeMap::new();

    for path in &self.packages {
      let package = Package::load(path).context(error::PackageLoad { path })?;
      packages.insert(package.hash, package);
    }

    if self.open {
      let url = format!("http://{}/", self.address);
      open::that(&url).context(error::Open { url: &url })?;
    }

    Runtime::new().context(error::Runtime)?.block_on(async {
      let node = Arc::new(
        Node::new(self.address, packages, 0)
          .await
          .context(error::NodeInitialize)?,
      );

      let clone = node.clone();
      tokio::spawn(async move {
        if let Some(bootstrap) = self.bootstrap {
          if let Err(err) = clone.ping(bootstrap).await {
            eprintln!("update error: {err}");
          }
        }

        if let Err(err) = clone.run().await {
          eprintln!("node error: {err}");
        }
      });

      axum_server::Server::bind((self.address, self.http_port).into())
        .serve(
          Router::new()
            .route("/", get(Self::node))
            .route("/favicon.ico", get(Self::favicon))
            .route("/node", get(Self::node))
            .route("/peer/:peer", get(Self::peer))
            .route("/static/*path", get(Self::static_asset))
            .route("/:package", get(Self::package))
            .route("/:package/:file", get(Self::file))
            .layer(Extension(node))
            .into_make_service(),
        )
        .await
        .context(error::Serve {
          address: (self.address, self.http_port),
        })
    })?;

    Ok(())
  }

  async fn favicon() -> ServerResult<Response> {
    Self::static_asset(Path("favicon.png".into())).await
  }

  async fn peer(
    node: Extension<Arc<Node>>,
    peer: Path<DeserializeFromStr<Id>>,
  ) -> ServerResult<PageHtml<SearchHtml>> {
    let peer = **peer;

    let hashes = node
      .search(peer)
      .await
      .map_err(|source| ServerError::Node { source })?
      .with_context(|| server_error::NotFound {
        message: format!("peer {peer} not found"),
      })?;

    let mut manifests = BTreeMap::new();

    for hash in hashes {
      let manifest = node
        .get(peer, hash)
        .await
        .map_err(|source| ServerError::Node { source })?;

      let Some(manifest) = manifest else {
        todo!();
      };

      manifests.insert(hash, manifest);
    }

    Ok(PageHtml {
      packages: node.packages.clone(),
      main: SearchHtml { peer, manifests },
    })
  }

  async fn static_asset(Path(path): Path<String>) -> ServerResult<Response> {
    let content = StaticAssets::get(if let Some(stripped) = path.strip_prefix('/') {
      stripped
    } else {
      &path
    })
    .with_context(|| server_error::NotFound {
      message: format!("{path} not found"),
    })?;

    let body = Body::from(content.data);
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Ok(
      Response::builder()
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(body)
        .unwrap(),
    )
  }

  async fn node(node: Extension<Arc<Node>>) -> PageHtml<NodeHtml> {
    PageHtml {
      packages: node.packages.clone(),
      main: NodeHtml {
        local: node.local.read().await.keys().copied().collect(),
        peer: node.peer(),
        received: node.received.load(atomic::Ordering::Relaxed),
        sent: node.sent.load(atomic::Ordering::Relaxed),
      },
    }
  }

  async fn package(
    node: Extension<Arc<Node>>,
    Path(DeserializeFromStr(package)): Path<DeserializeFromStr<Hash>>,
  ) -> ServerResult<PageHtml<PackageHtml>> {
    let package = node
      .packages
      .get(&package)
      .ok_or_else(|| ServerError::NotFound {
        message: format!("package {package} not found"),
      })?;

    Ok(PageHtml {
      packages: node.packages.clone(),
      main: PackageHtml {
        package: Arc::new(package.clone()),
      },
    })
  }

  async fn file(
    node: Extension<Arc<Node>>,
    Path((DeserializeFromStr(package), file)): Path<(DeserializeFromStr<Hash>, String)>,
  ) -> ServerResult {
    let package = node
      .packages
      .get(&package)
      .ok_or_else(|| ServerError::NotFound {
        message: format!("package {package} not found"),
      })?;

    match package.file(&file) {
      Some((content_type, content)) => Ok(Resource::new(content_type, content)),
      None => Err(ServerError::NotFound {
        message: format!("{file} not found"),
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, std::net::Ipv4Addr};

  #[test]
  fn package_load_error() {
    let tempdir = tempdir();

    let package = tempdir.join("app.package");

    assert_matches!(
      Server {
        address: Ipv4Addr::new(0, 0, 0, 0).into(),
        bootstrap: None,
        http_port: 80,
        open: false,
        packages: vec![package.clone()],
      }
      .run()
      .unwrap_err(),
      Error::PackageLoad { path, .. }
      if path == package,
    );
  }
}
