use super::*;

#[derive(Boilerplate)]
#[boilerplate(filename = "library.html")]
pub struct Library {
  pub packages: Vec<(Hash, Manifest)>,
  pub handlers: BTreeMap<Target, Hash>,
  pub root: ShadowRoot,
  api: Api,
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

  async fn search(
    self: Arc<Self>,
    section: HtmlElement,
    iframe: HtmlIFrameElement,
  ) -> Result<JsValue, JsValue> {
    SearchHtml.render(&section);
    let input = section.select::<HtmlInputElement>("input");
    let div = section.select::<HtmlDivElement>("div");
    input.add_event_listener("input", move |event: InputEvent| -> Promise {
      let div = div.clone();
      let arc = self.clone();
      wasm_bindgen_futures::future_to_promise(async move {
        let value = event.target().unwrap().cast::<HtmlInputElement>().value();
        log::info!("{value}");
        if let Ok(hash) = value.trim().parse() {
          match Api::default().search(hash).await? {
            Some(peer) => {
              div.set_inner_text(&peer.to_string());
              Api::default().bookmark(peer).await?;
              arc.update_bookmarks().await?;
            }
            None => div.set_inner_text("node not found"),
          }
        }
        Ok(JsValue::UNDEFINED)
      })
    });
    show_system(&section, &iframe);
    Ok(JsValue::UNDEFINED)
  }

  async fn update_bookmarks(self: Arc<Self>) -> Result<(), Error> {
    self.api.bookmarks().await?;
    Ok(())
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
      api,
    })
  }

  fn connected(self) {
    let arc = Arc::new(self);

    let iframe = arc.root.select::<HtmlIFrameElement>("iframe");
    let section = arc.root.select::<HtmlElement>("section");

    let origin = web_sys::window().unwrap().location().origin().unwrap();
    for button in arc.root.select_some::<HtmlButtonElement>("button.package") {
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
      arc
        .root
        .select::<HtmlButtonElement>("button#node")
        .add_event_listener("click", move |_: PointerEvent| -> Promise {
          wasm_bindgen_futures::future_to_promise(Self::node(section.clone(), iframe.clone()))
        });
    }

    {
      let iframe = iframe.clone();
      let section = section.clone();
      let clone = arc.clone();
      arc
        .root
        .select::<HtmlButtonElement>("button#search")
        .add_event_listener("click", move |_: PointerEvent| -> Promise {
          wasm_bindgen_futures::future_to_promise(
            arc.clone().search(section.clone(), iframe.clone()),
          )
        });
    }
  }
}
