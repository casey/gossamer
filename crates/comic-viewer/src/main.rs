use {
  self::comic::Comic,
  hypermedia::{
    boilerplate::Boilerplate,
    html_escaper::Escape,
    log,
    wasm_bindgen::{self, prelude::wasm_bindgen, JsValue},
    wasm_bindgen_futures, Component,
  },
  media::Media,
};

mod comic;

#[wasm_bindgen(main)]
async fn main() -> Result<(), JsValue> {
  hypermedia::initialize_console(log::Level::Trace)?;
  Comic::define();
  Ok(())
}
