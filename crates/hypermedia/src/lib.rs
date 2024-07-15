#![allow(async_fn_in_trait, clippy::result_large_err)]

use {
  media::{Hash, Id, Manifest, Target},
  reqwest::{StatusCode, Url},
  serde::de::DeserializeOwned,
  snafu::{ensure, OptionExt, ResultExt, Snafu},
  std::{
    collections::BTreeMap,
    io::{self, Cursor},
  },
  wasm_bindgen::{JsCast, JsError, JsValue},
  web_sys::HtmlBodyElement,
};

pub use {
  self::{api::Api, cast::Cast, error::Error},
  js_sys, log, media, wasm_bindgen, wasm_bindgen_futures, web_sys,
};

type Result<T = (), E = Error> = std::result::Result<T, E>;

mod api;
mod cast;
mod error;
mod macros;

pub fn body() -> Result<HtmlBodyElement> {
  Ok(
    web_sys::window()
      .context(error::WindowMissing)?
      .document()
      .expect("document missing")
      .body()
      .context(error::BodyMissing)?
      .cast(),
  )
}

pub fn initialize_console(level: log::Level) -> Result<(), Error> {
  console_error_panic_hook::set_once();
  console_log::init_with_level(level).map_err(|source| error::SetLogger { source }.build())?;
  Ok(())
}
