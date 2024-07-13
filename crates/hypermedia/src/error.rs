use super::*;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)), context(suffix(false)))]
pub enum Error {
  SetLogger {
    #[snafu(source(false))]
    source: log::SetLoggerError,
  },
}

impl From<Error> for JsValue {
  fn from(err: Error) -> Self {
    JsError::new(&err.to_string()).into()
  }
}
