use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum ServerError {
  NotFound { message: String },
  Node { source: node::Error },
}

impl IntoResponse for ServerError {
  fn into_response(self) -> Response {
    match self {
      Self::NotFound { message } => (StatusCode::NOT_FOUND, message).into_response(),
      Self::Node { .. } => {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
      }
    }
  }
}
