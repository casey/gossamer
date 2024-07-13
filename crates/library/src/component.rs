use super::*;

pub trait Component: Display + Sized + 'static {
  fn name() -> &'static str;

  async fn initialize() -> Result<Self, JsValue>;

  fn define() {
    js::define(
      Self::name(),
      &wasm_bindgen_futures::future_to_promise(Self::callback()),
    );
  }

  async fn callback() -> Result<JsValue, JsValue> {
    let component = Self::initialize().await?;

    let parser = DomParser::new().unwrap();
    let html = component.to_string();
    let document = parser
      .parse_from_string(&html, SupportedType::TextHtml)
      .unwrap();

    let callback = Closure::once(move |root| Self::connected(&Arc::new(component), root));

    let array = Array::new();

    array.push(&document.into());
    array.push(&callback.into_js_value());

    Ok(array.into())
  }

  fn connected(self: &Arc<Self>, _root: ShadowRoot) {}
}
