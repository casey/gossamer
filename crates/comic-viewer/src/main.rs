use {
  self::comic::Comic,
  hypermedia::{
    boilerplate::Boilerplate,
    html_escaper::Escape,
    log,
    media::{Media, Target},
    wasm_bindgen::{self, prelude::wasm_bindgen, JsValue},
    wasm_bindgen_futures,
    web_sys::{HtmlElement, ShadowRoot},
    Api, Component, Error,
  },
};

mod comic;

#[wasm_bindgen(main)]
async fn main() -> Result<(), JsValue> {
  hypermedia::initialize_console(log::Level::Trace)?;
  Comic::define();
  Ok(())
}
