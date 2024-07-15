use super::*;

#[derive(Boilerplate)]
#[boilerplate(filename = "app.html")]
pub struct App {
  name: String,
  paths: BTreeMap<String, Hash>,
  target: Target,
}

impl Component for App {
  fn name() -> &'static str {
    "media-app"
  }

  async fn initialize(_element: HtmlElement, _root: ShadowRoot) -> Result<Self, Error> {
    let api = Api::default();

    let manifest = api.manifest().await?;

    let Media::App { target, paths } = manifest.media else {
      return Err(Error::ContentType {
        content: manifest.ty(),
        target: Target::App,
      });
    };

    Ok(Self {
      target,
      paths,
      name: manifest.name,
    })
  }
}
