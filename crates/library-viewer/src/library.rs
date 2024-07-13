use super::*;

#[derive(Boilerplate)]
#[boilerplate(filename = "library.html")]
pub struct Library {
  pub packages: Vec<(Hash, Manifest)>,
  pub handlers: BTreeMap<Target, Hash>,
}

impl Component for Library {
  fn name() -> &'static str {
    "media-library"
  }

  async fn initialize() -> Result<Self, JsValue> {
    let api = media::Api::default();

    let handlers = api.handlers().await?;

    let mut packages = api
      .packages()
      .await?
      .into_iter()
      .collect::<Vec<(Hash, Manifest)>>();

    packages.sort_by(|x, y| x.1.name.cmp(&y.1.name));

    Ok(Self { packages, handlers })
  }

  fn connected(self: &Arc<Self>, root: ShadowRoot) {
    let iframe = root.select::<HtmlIFrameElement>("iframe");
    let origin = web_sys::window().unwrap().location().origin().unwrap();
    for button in root.select_all::<HtmlButtonElement>("button") {
      let dataset = button.dataset();
      let handler = dataset.get("handler").unwrap();
      let package = dataset.get("package").unwrap();
      let src = format!("{origin}/{handler}/{package}/");
      let iframe = iframe.clone();
      button.add_event_listener("click", move |_: PointerEvent| {
        if iframe.src() != src {
          iframe.set_src(&src);
        }
      });
    }
  }
}
