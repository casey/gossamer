use {
  self::app::App,
  hypermedia::{
    boilerplate::Boilerplate,
    html_escaper::Escape,
    log,
    wasm_bindgen::{self, prelude::wasm_bindgen, JsValue},
    wasm_bindgen_futures, Component,
  },
  media::{Hash, Media, Target},
  std::collections::BTreeMap,
};

mod app;

#[wasm_bindgen(main)]
async fn main() -> Result<(), JsValue> {
  hypermedia::initialize_console(log::Level::Trace)?;
  App::define();
  Ok(())
}
