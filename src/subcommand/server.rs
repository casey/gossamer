use {self::server_error::ServerError, super::*};

mod server_error;

#[derive(Parser)]
pub struct Server {
  #[arg(long, help = "Listen on <ADDRESS> for incoming requests.")]
  address: SocketAddr,
  #[arg(
    long,
    help = "Serve contents with app <PACKAGE>.",
    value_name = "PACKAGE"
  )]
  app: Utf8PathBuf,
  #[arg(long, help = "Serve contents of <PACKAGE>.", value_name = "PACKAGE")]
  content: Utf8PathBuf,
}

#[derive(Debug)]
struct State {
  app: Package,
  content: Package,
}

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

type ServerResult = std::result::Result<Resource, ServerError>;

impl Server {
  pub fn run(self) -> Result {
    let app = Package::load(&self.app).context(error::PackageLoad { path: &self.app })?;
    let content = Package::load(&self.content).context(error::PackageLoad {
      path: &self.content,
    })?;

    match app.manifest {
      Manifest::App { handles, .. } => {
        ensure!(
          content.manifest.ty() == handles,
          error::ContentType {
            content: content.manifest.ty(),
            handles,
          }
        );
      }
      _ => {
        return error::AppType {
          ty: app.manifest.ty(),
        }
        .fail()
      }
    }

    Runtime::new().context(error::Runtime)?.block_on(async {
      axum_server::Server::bind(self.address)
        .serve(
          Router::new()
            .route("/", get(Self::root))
            .route("/api/manifest", get(Self::manifest))
            .route("/app/*path", get(Self::app))
            .route("/content/*path", get(Self::content))
            .layer(Extension(Arc::new(State { app, content })))
            .into_make_service(),
        )
        .await
        .context(error::Serve {
          address: self.address,
        })
    })?;

    Ok(())
  }

  async fn manifest(Extension(state): Extension<Arc<State>>) -> Resource {
    Resource::new(
      mime::APPLICATION_JSON,
      serde_json::to_vec(&state.content.manifest).unwrap(),
    )
  }

  async fn root(Extension(state): Extension<Arc<State>>) -> ServerResult {
    Self::file(&state.app, "", "index.html")
  }

  async fn app(Extension(state): Extension<Arc<State>>, Path(path): Path<String>) -> ServerResult {
    Self::file(&state.app, "/app/", &path)
  }

  async fn content(
    Extension(state): Extension<Arc<State>>,
    Path(path): Path<String>,
  ) -> ServerResult {
    Self::file(&state.content, "/content/", &path)
  }

  fn file(package: &Package, prefix: &str, path: &str) -> ServerResult {
    match package.file(path) {
      Some((content_type, content)) => Ok(Resource::new(content_type, content)),
      None => Err(ServerError::NotFound {
        path: format!("{prefix}{path}"),
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

  #[test]
  fn app_load_error() {
    let tempdir = tempdir();

    let app = tempdir.path_utf8().join("app.package");
    let content = tempdir.path_utf8().join("content.package");

    assert_matches!(
      Server {
        address: "0.0.0.0:80".parse().unwrap(),
        app: app.clone(),
        content,
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
        app: app_package(),
        content: content.clone(),
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
        app: content_package(),
        content: content_package(),
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
        app: app_package(),
        content: app_package(),
      }
      .run()
      .unwrap_err(),
      Error::ContentType {
        content: Type::App,
        handles: Type::Comic,
        ..
      }
    );
  }

  // todo:
  // - check route content types
  //
  // - / -> index.html
  // - /api/manifest
  // - /app/*path
  // - /content/*path
  //
  // - fix backtrace in unwrap and assert_matches in test
  // - reorganize everything
}
