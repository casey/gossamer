use super::*;

pub trait Component: Display + Sized + 'static {
  fn name() -> &'static str;

  fn define(self) {
    let parser = DomParser::new().unwrap();
    let html = self.to_string();
    let document = parser
      .parse_from_string(&html, SupportedType::TextHtml)
      .unwrap();

    let component = Arc::new(self);
    let closure =
      Closure::wrap(Box::new(move |root| component.connected(root)) as Box<dyn FnMut(ShadowRoot)>);
    js::define(Self::name(), document, &closure);
    closure.forget();
  }

  fn connected(self: &Arc<Self>, _root: ShadowRoot) {}
}
