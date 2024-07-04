use {
  self::target_validator::TargetValidator,
  super::*,
  axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
  },
  tokio::runtime::Runtime,
  tower_http::{
    propagate_header::PropagateHeaderLayer,
    set_header::SetRequestHeaderLayer,
    validate_request::{ValidateRequest, ValidateRequestHeaderLayer},
  },
};

mod target_validator;

#[derive(Parser)]
pub struct Server {
  #[arg(long, help = "Listen on <ADDRESS> for incoming requests.")]
  address: SocketAddr,
  #[arg(long, help = "Load <PACKAGE> into library.", value_name = "<PACKAGE>", num_args = 0..)]
  packages: Vec<Utf8PathBuf>,
  #[arg(long, help = "Open server in browser.")]
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
  fn into_response(self) -> Response<Body> {
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
      axum_server::Server::bind(self.address)
        .serve(
          Router::new()
            .route("/", get(Self::library))
            .route("/:app/:content/", get(Self::root))
            .route("/:app/:content/api/manifest", get(Self::manifest))
            .route("/:app/:content/app/*path", get(Self::app))
            .route("/:app/:content/content/*path", get(Self::content))
            .layer(PropagateHeaderLayer::new(header::CONTENT_SECURITY_POLICY))
            .layer(SetRequestHeaderLayer::overriding(
              header::CONTENT_SECURITY_POLICY,
              move |request: &http::Request<Body>| {
                Some(Self::content_security_policy(self.address, request.uri()))
              },
            ))
            .layer(ValidateRequestHeaderLayer::custom(TargetValidator(
              library.clone(),
            )))
            .layer(Extension(library))
            .into_make_service(),
        )
        .await
        .context(error::Serve {
          address: self.address,
        })
    })?;

    Ok(())
  }

  fn content_security_policy(address: SocketAddr, uri: &Uri) -> HeaderValue {
    static RE: Lazy<Regex> = lazy_regex!("^/([[:xdigit:]]{64})/([[:xdigit:]]{64})/(app/.*)?$");

    let path = uri.path();

    if path == "/" {
      return HeaderValue::from_static("default-src 'self'");
    }

    RE.captures(path)
      .map(|captures| {
        HeaderValue::try_from(format!(
          "default-src http://{address}/{}/{}/",
          &captures[1], &captures[2],
        ))
        .unwrap()
      })
      .unwrap_or_else(|| HeaderValue::from_static("default-src"))
  }

  fn package(library: &Library, hash: Hash) -> ServerResult<&Package> {
    library.package(hash).ok_or_else(|| ServerError::NotFound {
      message: format!("package {hash} not found"),
    })
  }

  async fn library(library: Extension<Arc<Library>>) -> ServerResult {
    Self::file(
      library
        .handler(Target::Library)
        .ok_or_else(|| ServerError::NotFound {
          message: format!("library handler not found"),
        })?,
      "",
      "index.html",
    )
  }

  async fn root(library: Extension<Arc<Library>>, Path((app, _content)): HashRoot) -> ServerResult {
    Self::file(Self::package(&library, app.0)?, "", "index.html")
  }

  async fn manifest(
    library: Extension<Arc<Library>>,
    Path((_app, content)): HashRoot,
  ) -> ServerResult {
    Ok(Resource::new(
      mime::APPLICATION_JSON,
      serde_json::to_vec(&Self::package(&library, content.0)?.manifest).unwrap(),
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

  fn app_package() -> Utf8PathBuf {
    packages().join("app.package")
  }

  #[test]
  fn app_load_error() {
    let tempdir = tempdir();

    let app = tempdir.path_utf8().join("app.package");
    let content = tempdir.path_utf8().join("content.package");

    assert_matches!(
      Server {
        address: "0.0.0.0:80".parse().unwrap(),
        packages: vec![app.clone(), content],
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
        packages: vec![app_package(), content.clone()],
        open: false,
      }
      .run()
      .unwrap_err(),
      Error::PackageLoad { path, .. }
      if path == content,
    );
  }

  #[tokio::test]
  async fn routes() {
    fn hash_path(app: Hash, content: Hash, path: String) -> HashPath {
      Path((DeserializeFromStr(app), DeserializeFromStr(content), path))
    }

    let app_package = Package::load(&packages().join("app.package")).unwrap();
    let content_package = Package::load(&packages().join("content.package")).unwrap();
    let library_package = Package::load(&packages().join("library.package")).unwrap();

    let app = app_package.hash;
    let content = content_package.hash;

    let mut library = Library::default();

    library.add(app_package);
    library.add(content_package);
    library.add(library_package);

    let library = Extension(Arc::new(library));

    {
      let library = Server::library(library.clone()).await.unwrap();
      assert_eq!(library.content_type, mime::TEXT_HTML);
      assert!(str::from_utf8(&library.content)
        .unwrap()
        .contains("Library!"));
    }

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

  #[test]
  fn content_security_policy() {
    assert_eq!(
      Server::content_security_policy("0.0.0.0:80".parse().unwrap(), &Uri::from_static("/")),
      "default-src 'self'"
    );

    let app = blake3::hash(b"app");
    let content = blake3::hash(b"content");

    assert_eq!(
      Server::content_security_policy(
        "0.0.0.0:80".parse().unwrap(),
        &format!("/{app}/{content}/").parse().unwrap()
      ),
      format!("default-src http://0.0.0.0:80/{app}/{content}/"),
    );

    assert_eq!(
      Server::content_security_policy("0.0.0.0:80".parse().unwrap(), &"/foo".parse().unwrap()),
      "default-src",
    );
  }
}
