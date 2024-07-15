use {super::*, wasm_bindgen::prelude::wasm_bindgen};

#[wasm_bindgen(module = "/js/define.js")]
extern "C" {
  pub fn define(name: &str, callback: &Closure<dyn Fn(HtmlElement) -> Promise>);
}
