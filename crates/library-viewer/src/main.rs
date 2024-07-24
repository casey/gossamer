use {
  self::{
    library::Library,
    templates::{NodeHtml, SearchHtml},
  },
  hypermedia::{
    boilerplate::Boilerplate,
    html_escaper::Escape,
    js_sys::Promise,
    log,
    media::{Hash, Manifest, Target, Type},
    wasm_bindgen::{self, prelude::wasm_bindgen, JsValue},
    wasm_bindgen_futures,
    web_sys::{
      self, HtmlButtonElement, HtmlDivElement, HtmlElement, HtmlIFrameElement, HtmlInputElement,
      InputEvent, PointerEvent, ShadowRoot,
    },
    Api, Cast, Component, Error, EventTargetExt, SelectDocumentFragment, SelectElement,
  },
  std::{collections::BTreeMap, fmt::Display},
};

#[allow(unused)]
use hypermedia::debug;

mod library;
mod templates;

#[wasm_bindgen(main)]
async fn main() -> Result<(), JsValue> {
  hypermedia::initialize_console(log::Level::Trace)?;
  Library::define();
  Ok(())
}
