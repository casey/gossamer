use super::*;

#[wasm_bindgen(module = "/js/define.js")]
extern "C" {
  pub fn define(name: &str, connected: &Promise);
}