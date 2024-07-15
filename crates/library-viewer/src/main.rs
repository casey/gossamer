use {
  self::{state::State, view::View},
  hypermedia::{
    log,
    media::{api, Hash, Id, Manifest, Target, Type},
    wasm_bindgen::{self, prelude::wasm_bindgen, JsValue},
    Api,
  },
  std::collections::BTreeMap,
  xilem_web::{
    concurrent::memoized_await, core::fork, elements::html, interfaces::Element, App, DomFragment,
    DomView,
  },
};

mod state;
mod view;

#[wasm_bindgen(main)]
fn main() -> Result<(), JsValue> {
  hypermedia::initialize_console(log::Level::Trace)?;
  App::new(hypermedia::body()?, State::default(), State::update).run();
  Ok(())
}
