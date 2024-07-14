use super::*;

pub trait Component: Display + Sized + 'static {
  fn name() -> &'static str;

  async fn initialize() -> Result<Self, Error>;

  fn define() {
    js::define(
      Self::name(),
      &wasm_bindgen_futures::future_to_promise(Self::callback()),
    );
  }

  async fn callback() -> Result<JsValue, JsValue> {
    let parser = DomParser::new().unwrap();

    let component = match Self::initialize().await {
      Ok(component) => component,
      Err(err) => {
        log::error!("error initializing <{}>: {err}", Self::name());
        let document = parser
          .parse_from_string(&Dialog(err).to_string(), SupportedType::TextHtml)
          .unwrap();
        return Err(document.into());
      }
    };

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
