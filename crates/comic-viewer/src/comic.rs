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

  async fn initialize(_element: HtmlElement, _root: ShadowRoot) -> Result<Self, Error> {
    let api = Api::default();

    let manifest = api.manifest().await?;

    let Media::Comic { pages } = manifest.media else {
      return Err(Error::ContentType {
        content: manifest.ty(),
        target: Target::Comic,
      });
    };

    Ok(Self { pages: pages.len() })
  }
}
