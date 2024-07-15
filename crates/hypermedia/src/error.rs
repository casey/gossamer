use super::*;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)), context(suffix(false)))]
pub enum Error {
  SetLogger {
    #[snafu(source(false))]
    source: log::SetLoggerError,
  },
  #[snafu(display("deserializing response from {url} failed"))]
  Deserialize {
    url: Url,
    source: ciborium::de::Error<io::Error>,
  },
  #[snafu(display("request to {url} failed"))]
  Request {
    url: Url,
    source: reqwest::Error,
  },
  #[snafu(display("response from {url} failed with {status}"))]
  Status {
    url: Url,
    status: StatusCode,
  },
  WindowMissing,
  BodyMissing,
}

impl From<Error> for JsValue {
  fn from(err: Error) -> Self {
    JsError::new(&err.to_string()).into()
  }
}
