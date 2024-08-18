use {
  self::{server_error::ServerError, target_validator::TargetValidator},
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
  tower_http::{
    propagate_header::PropagateHeaderLayer,
    set_header::SetRequestHeaderLayer,
    validate_request::{ValidateRequest, ValidateRequestHeaderLayer},
  },
};

// todo:
// - okay_or_not_found

mod server_error;
mod target_validator;

#[derive(RustEmbed)]
#[folder = "static"]
struct StaticAssets;

struct Cbor<T>(T);

impl<T: Serialize> IntoResponse for Cbor<T> {
  fn into_response(self) -> Response {
    let mut cbor = Vec::new();
    ciborium::into_writer(&self.0, &mut cbor).unwrap();

    ([(header::CONTENT_TYPE, "application/cbor")], cbor).into_response()
  }
}

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

type HashPath = Path<(DeserializeFromStr<Hash>, DeserializeFromStr<Hash>, String)>;
type HashRoot = Path<(DeserializeFromStr<Hash>, DeserializeFromStr<Hash>)>;

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
    let mut library = Library::default();

    for path in &self.packages {
      let package = Package::load(path).context(error::PackageLoad { path })?;
      library.add(package);
    }

    if self.open {
      let url = format!("http://{}/", self.address);
      open::that(&url).context(error::Open { url: &url })?;
    }

    let library = Arc::new(library);

    Runtime::new().context(error::Runtime)?.block_on(async {
      let node = Arc::new(
        Node::new(self.address, library.clone(), 0)
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
            // old:
            .route("/api/handlers", get(Self::handlers))
            .route("/api/node", get(Self::node))
            .route("/api/packages", get(Self::packages))
            .route("/api/search/:peer", get(Self::search))
            .route("/app/*path", get(Self::root_app))
            .route("/:app/:content/", get(Self::app_root))
            .route("/:app/:content/api/manifest", get(Self::manifest))
            .route("/:app/:content/app/*path", get(Self::app))
            .route("/:app/:content/content/*path", get(Self::content))
            // .layer(PropagateHeaderLayer::new(header::CONTENT_SECURITY_POLICY))
            // .layer(SetRequestHeaderLayer::overriding(
            //   header::CONTENT_SECURITY_POLICY,
            //   move |request: &http::Request<Body>| {
            //     Some(Self::content_security_policy(self.http_port, request.uri()))
            //   },
            // ))
            // .layer(ValidateRequestHeaderLayer::custom(TargetValidator(
            //   library.clone(),
            // )))
            .layer(Extension(library))
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

  fn content_security_policy(port: u16, uri: &Uri) -> HeaderValue {
    static APP: Lazy<Regex> = lazy_regex!("^/([[:xdigit:]]{64})/([[:xdigit:]]{64})/(app/.*)?$");
    static ROOT: Lazy<Regex> = lazy_regex!("^/(app/.*)?$");

    let path = uri.path();

    if ROOT.is_match(path) {
      return HeaderValue::from_static("default-src 'unsafe-eval' 'unsafe-inline' 'self'");
    }

    APP
      .captures(path)
      .map(|captures| {
        HeaderValue::try_from(format!(
          "default-src 'unsafe-eval' 'unsafe-inline' http://localhost:{port}/{}/{}/",
          &captures[1], &captures[2],
        ))
        .unwrap()
      })
      .unwrap_or_else(|| HeaderValue::from_static("default-src"))
  }

  fn package(library: &Library, hash: Hash) -> ServerResult<&Package> {
    library
      .package(hash)
      .with_context(|| server_error::NotFound {
        message: format!("package {hash} not found"),
      })
  }

  async fn favicon() -> ServerResult<Response> {
    Self::static_asset(Path("favicon.png".into())).await
  }

  async fn root(library: Extension<Arc<Library>>) -> templates::Root {
    templates::Root {
      peer: None,
      node: None,
      library: (*library).clone(),
      package: None,
    }
  }

  async fn packages(library: Extension<Arc<Library>>) -> Cbor<BTreeMap<Hash, Manifest>> {
    Cbor(
      library
        .packages()
        .iter()
        .map(|(hash, package)| (*hash, package.manifest.clone()))
        .collect(),
    )
  }

  async fn peer(
    library: Extension<Arc<Library>>,
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
      library: (*library).clone(),
      package: None,
      peer: Some((peer, manifests)),
    })
  }

  async fn search(
    node: Extension<Arc<Node>>,
    peer: Path<DeserializeFromStr<Id>>,
  ) -> ServerResult<Cbor<Option<BTreeMap<Hash, Manifest>>>> {
    let peer = **peer;

    let hashes = node
      .search(peer)
      .await
      .map_err(|source| ServerError::Node { source })?;

    let Some(hashes) = hashes else {
      return Ok(Cbor(None));
    };

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

    Ok(Cbor(Some(manifests)))
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

  async fn handlers(library: Extension<Arc<Library>>) -> Cbor<BTreeMap<Target, Hash>> {
    Cbor(library.handlers().clone())
  }

  async fn node_new(
    library: Extension<Arc<Library>>,
    node: Extension<Arc<Node>>,
  ) -> ServerResult<templates::Root> {
    Ok(templates::Root {
      peer: None,
      node: Some(node.info().await),
      library: (*library).clone(),
      package: None,
    })
  }

  async fn node(node: Extension<Arc<Node>>) -> Cbor<media::api::Node> {
    Cbor(node.info().await)
  }

  async fn root_app(library: Extension<Arc<Library>>, path: Path<String>) -> ServerResult {
    Self::file(
      library
        .handler(Target::Root)
        .ok_or_else(|| ServerError::NotFound {
          message: "library handler not found".into(),
        })?,
      "/app/",
      &path,
    )
  }

  async fn foo(
    library: Extension<Arc<Library>>,
    Path(DeserializeFromStr(package)): Path<DeserializeFromStr<Hash>>,
  ) -> ServerResult<templates::Root> {
    library
      .package(package)
      .ok_or_else(|| ServerError::NotFound {
        message: format!("package {package} not found"),
      })?;

    Ok(templates::Root {
      node: None,
      peer: None,
      library: (*library).clone(),
      package: Some(package),
    })
  }

  async fn bar(
    library: Extension<Arc<Library>>,
    Path((DeserializeFromStr(package), file)): Path<(DeserializeFromStr<Hash>, String)>,
  ) -> ServerResult {
    let package = library
      .package(package)
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

  async fn app_root(
    library: Extension<Arc<Library>>,
    Path((app, _content)): HashRoot,
  ) -> ServerResult {
    Self::file(Self::package(&library, app.0)?, "", "index.html")
  }

  async fn manifest(
    library: Extension<Arc<Library>>,
    Path((_app, content)): HashRoot,
  ) -> ServerResult<Cbor<Manifest>> {
    Ok(Cbor(Self::package(&library, content.0)?.manifest.clone()))
  }

  async fn app(
    library: Extension<Arc<Library>>,
    Path((app, _content, path)): HashPath,
  ) -> ServerResult {
    Self::file(Self::package(&library, app.0)?, "/app/", &path)
  }

  async fn content(
    library: Extension<Arc<Library>>,
    Path((_app, content, path)): HashPath,
  ) -> ServerResult {
    Self::file(Self::package(&library, content.0)?, "/content/", &path)
  }

  fn file(package: &Package, prefix: &str, path: &str) -> ServerResult {
    match package.file(path) {
      Some((content_type, content)) => Ok(Resource::new(content_type, content)),
      None => Err(ServerError::NotFound {
        message: format!("{prefix}{path} not found"),
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
