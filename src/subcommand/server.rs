use {
  self::server_error::ServerError,
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
            .route("/", get(Self::root))
            .route("/static/*path", get(Self::static_asset))
            .route("/favicon.ico", get(Self::favicon))
            .route("/node", get(Self::node_new))
            .route("/peer/:peer", get(Self::peer))
            .route("/:package", get(Self::foo))
            .route("/:package/:file", get(Self::bar))
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

  async fn root(node: Extension<Arc<Node>>) -> templates::Root {
    templates::Root {
      peer: None,
      node: None,
      packages: node.packages.clone(),
      package: None,
    }
  }

  async fn peer(
    node: Extension<Arc<Node>>,
    peer: Path<DeserializeFromStr<Id>>,
  ) -> ServerResult<templates::Root> {
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

    Ok(templates::Root {
      node: None,
      packages: node.packages.clone(),
      package: None,
      peer: Some((peer, manifests)),
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

  async fn node_new(node: Extension<Arc<Node>>) -> ServerResult<templates::Root> {
    Ok(templates::Root {
      peer: None,
      node: Some(node.info().await),
      packages: node.packages.clone(),
      package: None,
    })
  }

  async fn foo(
    node: Extension<Arc<Node>>,
    Path(DeserializeFromStr(package)): Path<DeserializeFromStr<Hash>>,
  ) -> ServerResult<templates::Root> {
    node
      .packages
      .get(&package)
      .ok_or_else(|| ServerError::NotFound {
        message: format!("package {package} not found"),
      })?;

    Ok(templates::Root {
      node: None,
      peer: None,
      packages: node.packages.clone(),
      package: Some(package),
    })
  }

  async fn bar(
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

  #[tokio::test]
  async fn routes() {
    fn hash_path(app: Hash, content: Hash, path: String) -> HashPath {
      Path((DeserializeFromStr(app), DeserializeFromStr(content), path))
    }

    let mut library = Library::default();

    library.add(PACKAGES.app().clone());
    library.add(PACKAGES.comic().clone());
    library.add(PACKAGES.root().clone());

    let library = Extension(Arc::new(library));

    let app = PACKAGES.app().hash;
    let comic = PACKAGES.comic().hash;

    let root = Server::root(library.clone()).await.unwrap();
    assert_eq!(root.content_type, mime::TEXT_HTML);
    assert!(str::from_utf8(&root.content)
      .unwrap()
      .contains("<title>Library</title>"));

    let root_app = Server::root_app(library.clone(), Path("index.html".into()))
      .await
      .unwrap();
    assert_eq!(root_app.content_type, mime::TEXT_HTML);
    assert!(str::from_utf8(&root_app.content)
      .unwrap()
      .contains("<title>Library</title>"));

    let packages = Server::packages(library.clone()).await;
    assert_eq!(
      packages.0,
      library
        .packages()
        .iter()
        .map(|(hash, package)| (*hash, package.manifest.clone()))
        .collect()
    );

    let handlers = Server::handlers(library.clone()).await;
    assert_eq!(&handlers.0, library.handlers());

    let favicon = Server::favicon(library.clone()).await.unwrap();
    assert_eq!(favicon.content_type, "image/x-icon");
    assert_eq!(&favicon.content[..4], b"\x89PNG");

    let app_root = Server::app_root(
      library.clone(),
      Path((DeserializeFromStr(app), DeserializeFromStr(comic))),
    )
    .await
    .unwrap();
    assert_eq!(app_root.content_type, mime::TEXT_HTML);
    assert!(str::from_utf8(&app_root.content)
      .unwrap()
      .contains("<title>Comic</title>"));

    let manifest = Server::manifest(
      library.clone(),
      Path((DeserializeFromStr(app), DeserializeFromStr(comic))),
    )
    .await
    .unwrap();
    assert_eq!(manifest.0, PACKAGES.comic().manifest);

    let index_js = Server::app(library.clone(), hash_path(app, comic, "index.js".into()))
      .await
      .unwrap();
    assert_eq!(index_js.content_type, mime::TEXT_JAVASCRIPT);
    assert!(
      index_js.content.starts_with(b"const response ="),
      "{}",
      String::from_utf8(index_js.content).unwrap()
    );

    let page0 = Server::content(library.clone(), hash_path(app, comic, "0".into()))
      .await
      .unwrap();
    assert_eq!(page0.content_type, mime::IMAGE_JPEG);
    assert!(
      page0.content.starts_with(b"\xff\xd8\xff\xe0\x00\x10JFIF"),
      "{}",
      String::from_utf8_lossy(&page0.content),
    );

    assert_matches!(
      Server::content(library.clone(), hash_path(app, comic, "foo".into()))
        .await
        .unwrap_err(),
      ServerError::NotFound { message } if message == "/content/foo not found",
    );

    assert_matches!(
      Server::app(library.clone(), hash_path(app, comic, "foo".into()))
        .await
        .unwrap_err(),
      ServerError::NotFound { message } if message == "/app/foo not found",
    );
  }

  #[test]
  fn content_security_policy() {
    let port = 1234;

    assert_eq!(
      Server::content_security_policy(port, &Uri::from_static("/")),
      "default-src 'unsafe-eval' 'unsafe-inline' 'self'"
    );

    let app = PACKAGES.app().hash;
    let content = PACKAGES.comic().hash;

    assert_eq!(
      Server::content_security_policy(port, &format!("/{app}/{content}/").parse().unwrap()),
      format!("default-src 'unsafe-eval' 'unsafe-inline' http://localhost:{port}/{app}/{content}/"),
    );

    assert_eq!(
      Server::content_security_policy(port, &"/foo".parse().unwrap()),
      "default-src",
    );
  }
}
