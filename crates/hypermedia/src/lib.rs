#![allow(async_fn_in_trait, clippy::result_large_err)]

use {
  self::dialog::Dialog,
  boilerplate::Boilerplate,
  html_escaper::Escape,
  js_sys::Promise,
  media::{Hash, Manifest, Peer, Target, Type},
  reqwest::{StatusCode, Url},
  serde::de::DeserializeOwned,
  snafu::{ensure, ResultExt, Snafu},
  std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    io::{self, Cursor},
    ops::Deref,
  },
  wasm_bindgen::{
    closure::Closure,
    convert::{FromWasmAbi, IntoWasmAbi},
    JsCast, JsError, JsValue,
  },
  web_sys::{
    DocumentFragment, DomParser, Element, EventTarget, HtmlElement, ShadowRoot, ShadowRootInit,
    ShadowRootMode, SupportedType,
  },
};

pub use {
  self::{
    api::Api,
    cast::Cast,
    component::Component,
    error::Error,
    event_target_ext::EventTargetExt,
    select::{SelectDocumentFragment, SelectElement},
  },
  boilerplate, html_escaper, js_sys, log, media, wasm_bindgen, wasm_bindgen_futures, web_sys,
};

type Result<T = (), E = Error> = std::result::Result<T, E>;

mod api;
mod cast;
mod component;
mod dialog;
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
