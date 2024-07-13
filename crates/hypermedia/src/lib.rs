#![allow(async_fn_in_trait)]

use {
  self::error::Error,
  js_sys::{Array, Promise},
  snafu::Snafu,
  std::{fmt::Display, ops::Deref, sync::Arc},
  wasm_bindgen::{closure::Closure, convert::FromWasmAbi, JsCast, JsError, JsValue},
  web_sys::{DocumentFragment, DomParser, EventTarget, ShadowRoot, SupportedType},
};

pub use {
  self::{cast::Cast, component::Component, event_target_ext::EventTargetExt, select::Select},
  boilerplate, html_escaper, js_sys, log, wasm_bindgen, wasm_bindgen_futures, web_sys,
};

mod cast;
mod component;
mod error;
mod event_target_ext;
mod js;
mod macros;
mod select;

pub fn initialize_console(level: log::Level) -> Result<(), Error> {
  console_error_panic_hook::set_once();
  console_log::init_with_level(level).map_err(|source| error::SetLogger { source }.build())?;
  Ok(())
}
