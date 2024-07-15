use super::*;

#[derive(Default)]
pub(crate) struct State {
  app: Option<App>,
}

impl State {
  pub(crate) fn update(&mut self) -> impl DomView<Self> {
    use html::*;

    match &self.app {
      None => fork(
        div("Loading…").boxed(),
        memoized_await(
          (),
          |()| async {
            let manifest = Api::default().manifest().await.unwrap();

            let Media::App { target, paths } = manifest.media else {
              todo!();
            };

            App {
              name: manifest.name,
              paths,
              target,
            }
          },
          |state: &mut State, app| state.app = Some(app),
        ),
      )
      .boxed(),
      Some(app) => div((
        h1(app.name.to_string()),
        div(format!("target: {}", app.target)),
        h2("Files"),
        ul(
          app
            .paths
            .iter()
            .map(|(path, hash)| li(format!("{path}: {hash}")))
            .collect::<Vec<_>>(),
        ),
      ))
      .boxed(),
    }
  }
}
