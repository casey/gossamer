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
    log::info!("node");
    let node = Api::default().node().await?;
    NodeHtml(node).render(&section);
    show_system(&section, &iframe);
    Ok(JsValue::UNDEFINED)
  }

  async fn search(section: HtmlElement, iframe: HtmlIFrameElement) -> Result<JsValue, JsValue> {
    SearchHtml.render(&section);
    let input = section.select::<HtmlInputElement>("input");
    let div = section.select::<HtmlDivElement>("div");
    input.add_event_listener("input", move |event: InputEvent| -> Promise {
      wasm_bindgen_futures::future_to_promise(async move {
        let value = event.target().unwrap().cast::<HtmlInputElement>().value();
        log::info!("{value}");
        if let Ok(hash) = value.parse::<Hash>() {
          let peer = Api::default().search(hash).await?;
          // - get peer info
          // - temporarily view and search peer
          // - add peer to sidebar
          log::info!("{peer:?}");
        }
        Ok(JsValue::UNDEFINED)
      })
    });
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

    {
      let iframe = iframe.clone();
      let section = section.clone();
      self
        .root
        .select::<HtmlButtonElement>("button#node")
        .add_event_listener("click", move |_: PointerEvent| -> Promise {
          wasm_bindgen_futures::future_to_promise(Self::node(section.clone(), iframe.clone()))
        });
    }

    {
      let iframe = iframe.clone();
      let section = section.clone();
      self
        .root
        .select::<HtmlButtonElement>("button#search")
        .add_event_listener("click", move |_: PointerEvent| -> Promise {
          wasm_bindgen_futures::future_to_promise(Self::search(section.clone(), iframe.clone()))
        });
    }
  }
}
