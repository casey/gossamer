use super::*;

#[derive(Boilerplate)]
#[boilerplate(filename = "library.html")]
pub struct Library {
  pub packages: Vec<(Hash, Manifest)>,
  pub handlers: BTreeMap<Target, Hash>,
  pub root: ShadowRoot,
}

fn show_system(section: &HtmlElement, iframe: &HtmlIFrameElement) {
  section.style().set_property("display", "block").unwrap();
  iframe.style().set_property("display", "none").unwrap();
}

fn show_app(section: &HtmlElement, iframe: &HtmlIFrameElement) {
  section.style().remove_property("display").unwrap();
  iframe.style().remove_property("display").unwrap();
}

impl Library {
  async fn node(section: HtmlElement, iframe: HtmlIFrameElement) -> Result<JsValue, JsValue> {
    let node = Api::default().node().await?;
    log::info!("{node:?}");
    NodeHtml(node).render(&section);
    show_system(&section, &iframe);
    Ok(JsValue::UNDEFINED)
  }
}

trait Render: Display {
  fn render(&self, element: &HtmlElement) {
    element.set_inner_html(&self.to_string());
  }
}

impl<T: Display> Render for T {}

#[derive(Boilerplate)]
struct NodeHtml(hypermedia::media::api::Node);

impl Component for Library {
  fn name() -> &'static str {
    "media-library"
  }

  async fn initialize(_element: HtmlElement, root: ShadowRoot) -> Result<Self, Error> {
    let api = Api::default();

    let handlers = api.handlers().await?;

    let mut packages = api
      .packages()
      .await?
      .into_iter()
      .collect::<Vec<(Hash, Manifest)>>();

    packages.sort_by(|x, y| x.1.name.cmp(&y.1.name));

    Ok(Self {
      packages,
      handlers,
      root,
    })
  }

  fn connected(self) {
    let iframe = self.root.select::<HtmlIFrameElement>("iframe");
    let section = self.root.select::<HtmlElement>("section");

    let origin = web_sys::window().unwrap().location().origin().unwrap();
    for button in self.root.select_some::<HtmlButtonElement>("button.package") {
      let dataset = button.dataset();
      let handler = dataset.get("handler").unwrap();
      let package = dataset.get("package").unwrap();
      let src = format!("{origin}/{handler}/{package}/");
      let iframe = iframe.clone();
      let section = section.clone();
      button.add_event_listener("click", move |_: PointerEvent| {
        if iframe.src() != src {
          iframe.set_src(&src);
        }
        show_app(&section, &iframe);
      });
    }

    self
      .root
      .select::<HtmlButtonElement>("button#node")
      .add_event_listener("click", move |_: PointerEvent| -> Promise {
        wasm_bindgen_futures::future_to_promise(Self::node(section.clone(), iframe.clone()))
      });
  }
}
