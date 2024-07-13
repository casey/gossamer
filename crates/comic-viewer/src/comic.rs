use super::*;

#[derive(Boilerplate)]
#[boilerplate(filename = "comic.html")]
pub struct Comic {
  pages: usize,
}

impl Component for Comic {
  fn name() -> &'static str {
    "media-comic"
  }

  async fn initialize() -> Result<Self, JsValue> {
    let api = Api::default();

    let manifest = api.manifest().await?;

    let Media::Comic { pages } = manifest.media else {
      panic!()
    };

    Ok(Self { pages: pages.len() })
  }
}
