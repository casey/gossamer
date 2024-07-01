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

impl Server {
  pub fn run(self) -> Result {
    let app = Package::load(&self.app)?;
    let content = Package::load(&self.content)?;

    match app.manifest {
      Manifest::App { handles, .. } => {
        if content.manifest.ty() != handles {
          return Err(Error::ContentType {
            backtrace: Backtrace::capture(),
            content: content.manifest.ty(),
            app: handles,
          });
        }
      }
      _ => {
        return Err(Error::AppType {
          backtrace: Backtrace::capture(),
          ty: app.manifest.ty(),
        })
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

  async fn manifest(Extension(state): Extension<Arc<State>>) -> impl IntoResponse {
    (
      [(header::CONTENT_TYPE, "application/json")],
      serde_json::to_string(&state.content.manifest).unwrap(),
    )
  }

  async fn root(Extension(state): Extension<Arc<State>>) -> impl IntoResponse {
    Self::file(&state.app, "", "index.html")
  }

  async fn app(
    Extension(state): Extension<Arc<State>>,
    Path(path): Path<String>,
  ) -> impl IntoResponse {
    Self::file(&state.app, "/app/", &path)
  }

  async fn content(
    Extension(state): Extension<Arc<State>>,
    Path(path): Path<String>,
  ) -> impl IntoResponse {
    Self::file(&state.content, "/content/", &path)
  }

  fn file(package: &Package, prefix: &str, path: &str) -> impl IntoResponse {
    match package.get(path) {
      Some((content_type, content)) => Ok(([(header::CONTENT_TYPE, content_type)], content)),
      None => Err(ServerError::NotFound {
        path: format!("{prefix}{path}"),
      }),
    }
  }
}
