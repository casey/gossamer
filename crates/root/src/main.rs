use {
  self::{
    cast::Cast, component::Component, ext::EventTargetExt, media_library::MediaLibrary,
    select::Select,
  },
  boilerplate::Boilerplate,
  html_escaper::Escape,
  js_sys::{Array, Promise},
  media::{Hash, Manifest, Target},
  std::{collections::BTreeMap, fmt::Display, ops::Deref, sync::Arc},
  wasm_bindgen::{closure::Closure, convert::FromWasmAbi, prelude::wasm_bindgen, JsCast, JsValue},
  web_sys::{
    DocumentFragment, DomParser, EventTarget, HtmlButtonElement, HtmlIFrameElement, PointerEvent,
    ShadowRoot, SupportedType,
  },
};

#[macro_use]
mod macros;

mod cast;
mod component;
mod ext;
mod js;
mod media_library;
mod select;

#[wasm_bindgen(main)]
async fn main() -> Result<(), JsValue> {
  console_error_panic_hook::set_once();
  console_log::init_with_level(log::Level::Trace).unwrap();
  MediaLibrary::define();
  Ok(())
}
