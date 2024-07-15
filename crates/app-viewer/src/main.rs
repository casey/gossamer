use {
  self::{app::App, state::State},
  hypermedia::{
    log,
    media::{Hash, Media, Target},
    wasm_bindgen::{self, prelude::wasm_bindgen, JsValue},
    wasm_bindgen_futures, Api,
  },
  std::collections::BTreeMap,
  xilem_web::{concurrent::memoized_await, core::fork, elements::html, DomView},
};

mod app;
mod state;

#[wasm_bindgen(main)]
async fn main() -> Result<(), JsValue> {
  hypermedia::initialize_console(log::Level::Trace)?;
  xilem_web::App::new(hypermedia::body()?, State::default(), State::update).run();
  Ok(())
}
