use super::*;

#[wasm_bindgen(module = "/js/define.js")]
extern "C" {
  pub fn define(name: &str, document: Document, connected: &Closure<dyn FnMut(ShadowRoot)>);
}
