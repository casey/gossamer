use super::*;

#[derive(Default)]
pub(crate) struct State {
  pages: Option<u64>,
}

impl State {
  pub(crate) fn update(&mut self) -> impl DomView<Self> {
    use html::*;

    match &self.pages {
      None => fork(
        div("Loading…").boxed(),
        memoized_await(
          (),
          |()| async {
            let manifest = Api::default().manifest().await.unwrap();

            let Media::Comic { pages } = manifest.media else {
              todo!();
            };

            pages.len() as u64
          },
          |state: &mut State, pages| state.pages = Some(pages),
        ),
      )
      .boxed(),
      Some(pages) => div(
        (0..*pages)
          .map(|page| img(()).src(format!("content/{page}")))
          .collect::<Vec<_>>(),
      )
      .boxed(),
    }
  }
}
