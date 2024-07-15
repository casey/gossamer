use super::*;

#[derive(Debug, Default)]
pub(crate) struct State {
  handlers: BTreeMap<Target, Hash>,
  packages: BTreeMap<Hash, Manifest>,
  view: View,
}

impl State {
  fn package_list(&mut self, apps: bool) -> impl DomView<Self> {
    use html::*;
    ul(
      self
        .packages
        .iter()
        .filter(|(_hash, manifest)| {
          if apps {
            manifest.ty() == Type::App
          } else {
            manifest.ty() != Type::App
          }
        })
        .map(|(content, manifest)| {
          li(
            if let Some(handler) = self.handlers.get(&manifest.ty().into()) {
              let handler = *handler;
              let content = *content;
              button(manifest.name.clone())
                .on_click(move |state: &mut State, _| {
                  state.view = View::Package { handler, content };
                })
                .boxed()
            } else {
              manifest.name.clone().boxed()
            },
          )
        })
        .collect::<Vec<_>>(),
    )
  }

  pub(crate) fn update(&mut self) -> impl DomFragment<Self> {
    use html::*;

    log::info!("update");

    (
      nav((
        h1("System"),
        ul(li(button("Node").on_click(|state: &mut State, _| {
          state.view = View::NodeRequest;
        }))),
        h1("Apps"),
        self.package_list(true),
        h1("Content"),
        self.package_list(false),
      )),
      fork(
        main(self.view()),
        memoized_await(
          (),
          |()| async {
            let handlers = Api::default().handlers().await.unwrap();

            let packages = Api::default().packages().await.unwrap();

            (handlers, packages)
          },
          |state: &mut State, output| (state.handlers, state.packages) = output,
        ),
      ),
    )
  }

  fn node(&self, node: &api::Node) -> impl DomView<State> {
    use html::*;

    div((
      h1("Node"),
      node.peer.to_string(),
      h2("Statistics"),
      h3("Sent"),
      node.sent,
      h3("Received"),
      node.received,
      h2("Peers"),
      ul(
        node
          .local
          .iter()
          .map(|&id| {
            button(id.to_string()).on_click(move |state: &mut State, _| {
              state.view = View::SearchRequest { id };
            })
          })
          .collect::<Vec<_>>(),
      ),
    ))
  }

  fn view(&self) -> impl DomView<State> {
    use html::*;

    match &self.view {
      View::Default => div(()).boxed(),
      View::NodeRequest => fork(
        div(()),
        memoized_await(
          (),
          |()| async { Api::default().node().await.unwrap() },
          |state: &mut State, node| state.view = View::NodeResponse { node },
        ),
      )
      .boxed(),
      View::NodeResponse { node } => self.node(node).boxed(),
      View::Package { handler, content } => iframe(())
        .attr("src", format!("/{handler}/{content}/"))
        .boxed(),
      View::SearchRequest { id } => fork(
        div(()),
        memoized_await(
          *id,
          |id| {
            let id = *id;
            async move { (id, Api::default().search(id).await.unwrap()) }
          },
          |state: &mut State, output| {
            state.view = View::SearchResponse {
              id: output.0,
              packages: output.1,
            }
          },
        ),
      )
      .boxed(),
      View::SearchResponse { id, packages } => div((
        h1(id.to_string()),
        if let Some(packages) = packages {
          ul(
            packages
              .iter()
              .map(|(_hash, manifest)| li(manifest.name.to_string()))
              .collect::<Vec<_>>(),
          )
          .boxed()
        } else {
          "Could not find peer".boxed()
        },
      ))
      .boxed(),
    }
  }
}
