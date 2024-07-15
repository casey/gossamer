use super::*;

pub trait Component: Display + Sized + 'static {
  fn name() -> &'static str;

  async fn initialize(element: HtmlElement, shadow: ShadowRoot) -> Result<Self, Error>;

  fn define() {
    js::define(
      Self::name(),
      &Closure::new(|element| Self::promise(element)),
    );
  }

  fn promise(element: HtmlElement) -> Promise {
    wasm_bindgen_futures::future_to_promise(Self::callback(element))
  }

  async fn callback(element: HtmlElement) -> Result<JsValue, JsValue> {
    let root = element
      .attach_shadow(&ShadowRootInit::new(ShadowRootMode::Closed))
      .unwrap();

    let parser = DomParser::new().unwrap();

    let component = match Self::initialize(element, root.clone()).await {
      Ok(component) => component,
      Err(err) => {
        let message = format!("error initializing <{}>: {err}", Self::name());
        let document = parser
          .parse_from_string(&Dialog(err).to_string(), SupportedType::TextHtml)
          .unwrap();
        root
          .append_child(&document.document_element().unwrap())
          .unwrap();
        return Err(message.into());
      }
    };

    let html = component.to_string();

    let document = parser
      .parse_from_string(&html, SupportedType::TextHtml)
      .unwrap();

    root
      .append_child(&document.document_element().unwrap())
      .unwrap();

    component.connected();

    Ok(JsValue::UNDEFINED)
  }

  fn connected(self) {}
}
