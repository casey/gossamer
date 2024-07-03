use super::*;

#[derive(Parser)]
#[clap(group(
  ArgGroup::new("mode")
    .required(true)
    .args(&["app", "library"]))
)]
pub struct Server {
  #[arg(long, help = "Listen on <ADDRESS> for incoming requests.")]
  address: SocketAddr,
  #[arg(
    long,
    help = "Serve contents with app <PACKAGE>.",
    value_name = "PACKAGE",
    requires = "content"
  )]
  app: Option<Utf8PathBuf>,
  #[arg(long, help = "Serve contents of <PACKAGE>.", value_name = "PACKAGE")]
  content: Option<Utf8PathBuf>,
  #[arg(
    long,
    help = "Serve contents of library in <DIRECTORY>.",
    value_name = "DIRECTORY",
    conflicts_with = "content"
  )]
  library: Option<Utf8PathBuf>,
  #[arg(long, help = "Load server in browser.")]
  open: bool,
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
  fn into_response(self) -> axum::http::Response<axum::body::Body> {
    (
      [(header::CONTENT_TYPE, self.content_type.to_string())],
      self.content,
    )
      .into_response()
  }
}

#[derive(Debug, PartialEq)]
pub enum ServerError {
  NotFound { message: String },
}

impl IntoResponse for ServerError {
  fn into_response(self) -> Response {
    match self {
      Self::NotFound { message } => (StatusCode::NOT_FOUND, message).into_response(),
    }
  }
}

type ServerResult<T = Resource> = std::result::Result<T, ServerError>;

impl Server {
  pub fn run(self) -> Result {
    fn load(path: &Utf8Path) -> Result<Package> {
      Package::load(path).context(error::PackageLoad { path })
    }

    let app = self.app.as_ref().map(|path| load(path)).transpose()?;
    let content = self.content.as_ref().map(|path| load(path)).transpose()?;

    if let (Some(app), Some(content)) = (&app, &content) {
      match app.manifest {
        Manifest::App { target, .. } => {
          let content = content.manifest.ty();
          ensure!(
            match target {
              Target::App => content == Type::App,
              Target::Library => todo!(),
              Target::Comic => content == Type::Comic,
            },
            error::Target { content, target }
          );
        }
        _ => {
          return error::AppType {
            ty: app.manifest.ty(),
          }
          .fail()
        }
      };

      let url = format!("http://{}/{}/{}/", self.address, app.hash, content.hash);

      if self.open {
        open::that(&url).context(error::Open { url: &url })?;
      }
    }

    let mut library = Library::default();

    app.map(|package| library.add(package));
    content.map(|package| library.add(package));

    Runtime::new().context(error::Runtime)?.block_on(async {
      axum_server::Server::bind(self.address)
        .serve(
          Router::new()
            .route("/:app/:content/", get(Self::root))
            .route("/:app/:content/api/manifest", get(Self::manifest))
            .route("/:app/:content/app/*path", get(Self::app))
            .route("/:app/:content/content/*path", get(Self::content))
            .layer(Extension(Arc::new(library)))
            .into_make_service(),
        )
        .await
        .context(error::Serve {
          address: self.address,
        })
    })?;

    Ok(())
  }

  fn package(library: &Library, hash: Hash) -> ServerResult<&Package> {
    library.package(hash).ok_or_else(|| ServerError::NotFound {
      message: format!("package {hash} not found"),
    })
  }

  async fn root(library: Extension<Arc<Library>>, hash_root: HashRoot) -> ServerResult {
    Self::file(
      Self::package(&library, hash_root.0 .0 .0)?,
      "",
      "index.html",
    )
  }

  async fn manifest(library: Extension<Arc<Library>>, hash_root: HashRoot) -> ServerResult {
    Ok(Resource::new(
      mime::APPLICATION_JSON,
      serde_json::to_vec(&Self::package(&library, hash_root.0 .1 .0)?.manifest).unwrap(),
    ))
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
  use super::*;

  static PACKAGES: Mutex<Option<TempDir>> = Mutex::new(None);

  fn packages() -> Utf8PathBuf {
    let mut packages = PACKAGES.lock().unwrap();

    if packages.is_none() {
      let tempdir = tempdir();

      subcommand::package::Package {
        root: "apps/comic".into(),
        output: tempdir.path_utf8().join("app.package"),
      }
      .run()
      .unwrap();

      subcommand::package::Package {
        root: "content/comic".into(),
        output: tempdir.path_utf8().join("content.package"),
      }
      .run()
      .unwrap();

      *packages = Some(tempdir);
    }

    packages.as_ref().unwrap().path_utf8().into()
  }

  fn content_package() -> Utf8PathBuf {
    packages().join("content.package")
  }

  fn app_package() -> Utf8PathBuf {
    packages().join("app.package")
  }

  fn hash_path(app: Hash, content: Hash, path: String) -> HashPath {
    Path((DeserializeFromStr(app), DeserializeFromStr(content), path))
  }

  #[test]
  fn app_load_error() {
    let tempdir = tempdir();

    let app = tempdir.path_utf8().join("app.package");
    let content = tempdir.path_utf8().join("content.package");

    assert_matches!(
      Server {
        address: "0.0.0.0:80".parse().unwrap(),
        app: Some(app.clone()),
        content: Some(content),
        library: None,
        open: false,
      }
      .run()
      .unwrap_err(),
      Error::PackageLoad { path, .. }
      if path == app,
    );
  }

  #[test]
  fn content_load_error() {
    let tempdir = tempdir();

    let content = tempdir.path_utf8().join("content.package");

    assert_matches!(
      Server {
        address: "0.0.0.0:80".parse().unwrap(),
        app: Some(app_package()),
        content: Some(content.clone()),
        library: None,
        open: false,
      }
      .run()
      .unwrap_err(),
      Error::PackageLoad { path, .. }
      if path == content,
    );
  }

  #[test]
  fn app_package_is_not_app() {
    assert_matches!(
      Server {
        address: "0.0.0.0:80".parse().unwrap(),
        app: Some(content_package()),
        content: Some(content_package()),
        library: None,
        open: false,
      }
      .run()
      .unwrap_err(),
      Error::AppType { ty, .. }
      if ty == Type::Comic,
    );
  }

  #[test]
  fn app_doesnt_handle_content_type() {
    assert_matches!(
      Server {
        address: "0.0.0.0:80".parse().unwrap(),
        app: Some(app_package()),
        content: Some(app_package()),
        library: None,
        open: false,
      }
      .run()
      .unwrap_err(),
      Error::Target {
        content: Type::App,
        target: Target::Comic,
        ..
      }
    );
  }

  #[tokio::test]
  async fn routes() {
    let app_package = Package::load(&packages().join("app.package")).unwrap();
    let content_package = Package::load(&packages().join("content.package")).unwrap();

    let app = app_package.hash;
    let content = content_package.hash;

    let mut library = Library::default();

    library.add(app_package);
    library.add(content_package);

    let library = Extension(Arc::new(library));

    let root = Server::root(
      library.clone(),
      Path((DeserializeFromStr(app), DeserializeFromStr(content))),
    )
    .await
    .unwrap();
    assert_eq!(root.content_type, mime::TEXT_HTML);
    assert!(root.content.starts_with(b"<html>"));

    let manifest = Server::manifest(
      library.clone(),
      Path((DeserializeFromStr(app), DeserializeFromStr(content))),
    )
    .await
    .unwrap();
    assert_eq!(manifest.content_type, mime::APPLICATION_JSON);
    assert!(
      manifest.content.starts_with(b"{\"type\":\"comic\""),
      "{}",
      String::from_utf8(manifest.content).unwrap()
    );

    let index_js = Server::app(library.clone(), hash_path(app, content, "index.js".into()))
      .await
      .unwrap();
    assert_eq!(index_js.content_type, mime::TEXT_JAVASCRIPT);
    assert!(
      index_js.content.starts_with(b"const response ="),
      "{}",
      String::from_utf8(index_js.content).unwrap()
    );

    let page0 = Server::content(library.clone(), hash_path(app, content, "0".into()))
      .await
      .unwrap();
    assert_eq!(page0.content_type, mime::IMAGE_JPEG);
    assert!(
      page0.content.starts_with(b"\xff\xd8\xff\xe0\x00\x10JFIF"),
      "{}",
      String::from_utf8_lossy(&page0.content),
    );

    assert_eq!(
      Server::content(library.clone(), hash_path(app, content, "foo".into()))
        .await
        .unwrap_err(),
      ServerError::NotFound {
        message: "/content/foo not found".into(),
      },
    );

    assert_eq!(
      Server::app(library.clone(), hash_path(app, content, "foo".into()))
        .await
        .unwrap_err(),
      ServerError::NotFound {
        message: "/app/foo not found".into(),
      },
    );
  }
}
