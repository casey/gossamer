use super::*;

#[derive(Boilerplate)]
#[boilerplate(filename = "app.html")]
pub struct App {
  name: String,
  target: Target,
  paths: BTreeMap<String, Hash>,
}

impl Component for App {
  fn name() -> &'static str {
    "media-app"
  }

  async fn initialize() -> Result<Self, Error> {
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
