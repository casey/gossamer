use super::*;

#[derive(Debug, PartialEq)]
pub enum ServerError {
  NotFound { path: String },
}

impl IntoResponse for ServerError {
  fn into_response(self) -> Response {
    match self {
      Self::NotFound { path } => {
        (StatusCode::NOT_FOUND, format!("{path} not found")).into_response()
      }
    }
  }
}
