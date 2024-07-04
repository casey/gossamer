use super::*;

#[derive(Boilerplate)]
#[boilerplate(filename = "media-library.html")]
pub struct MediaLibrary {
  pub packages: BTreeMap<Hash, Manifest>,
  pub handlers: BTreeMap<Target, Hash>,
}

impl MediaLibrary {
  pub async fn new() -> Result<Self, JsValue> {
    let api = media::Api::default();

    let packages = api.packages().await?;
    let handlers = api.handlers().await?;

    Ok(Self { packages, handlers })
  }
}

impl Component for MediaLibrary {
  fn name() -> &'static str {
    "media-library"
  }

  fn connected(self: &Arc<Self>, root: ShadowRoot) {
    let iframe = root.select::<HtmlIFrameElement>("iframe");
    let origin = web_sys::window().unwrap().location().origin().unwrap();
    for button in root.select_all::<HtmlButtonElement>("button") {
      let dataset = button.dataset();
      let src = format!(
        "{origin}/{}/{}/",
        dataset.get("handler").unwrap(),
        dataset.get("package").unwrap(),
      );
      let iframe = iframe.clone();
      button.add_event_listener("click", move |_: PointerEvent| {
        if iframe.src() != src {
          iframe.set_src(&src);
        }
      });
    }
  }
}
