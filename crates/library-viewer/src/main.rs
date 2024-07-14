use {
  self::library::Library,
  hypermedia::{
    boilerplate::Boilerplate,
    html_escaper::Escape,
    log,
    media::{Hash, Manifest, Target, Type},
    wasm_bindgen::{self, prelude::wasm_bindgen, JsValue},
    wasm_bindgen_futures,
    web_sys::{self, HtmlButtonElement, HtmlIFrameElement, PointerEvent, ShadowRoot},
    Api, Component, Error, EventTargetExt, Select,
  },
  std::{collections::BTreeMap, sync::Arc},
};

mod library;

#[wasm_bindgen(main)]
async fn main() -> Result<(), JsValue> {
  hypermedia::initialize_console(log::Level::Trace)?;
  Library::define();
  Ok(())
}
