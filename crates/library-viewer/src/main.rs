// use {
//   self::{
//     library::Library,
//     templates::{NodeHtml, SearchHtml},
//   },
//   hypermedia::{
//     boilerplate::Boilerplate,
//     html_escaper::Escape,
//     js_sys::Promise,
//     log,
//     media::{Hash, Manifest, Target, Type},
//     wasm_bindgen::{self, prelude::wasm_bindgen, JsValue},
//     wasm_bindgen_futures,
//     web_sys::{
//       self, HtmlButtonElement, HtmlDivElement, HtmlElement, HtmlIFrameElement, HtmlInputElement,
//       InputEvent, PointerEvent, ShadowRoot,
//     },
//     Api, Cast, Component, Error, EventTargetExt, SelectDocumentFragment, SelectElement,
//   },
//   std::{collections::BTreeMap, fmt::Display, sync::Arc},
// };

// #[allow(unused)]
// use hypermedia::debug;

// mod library;
// mod templates;

// #[wasm_bindgen(main)]
// async fn main() -> Result<(), JsValue> {
//   hypermedia::initialize_console(log::Level::Trace)?;
//   Library::define();
//   Ok(())
// }

use {
  hypermedia::{
    log,
    media::{api, Hash, Manifest, Target, Type},
    wasm_bindgen::{self, prelude::wasm_bindgen, JsValue},
    Api,
  },
  std::collections::BTreeMap,
  xilem_web::{
    concurrent::memoized_await, core::fork, elements::html, interfaces::Element, App, DomFragment,
    DomView,
  },
};

#[derive(Debug, Default)]
struct State {
  handlers: BTreeMap<Target, Hash>,
  packages: BTreeMap<Hash, Manifest>,
  view: View,
}

#[derive(Clone, Debug, Default, PartialEq)]
enum View {
  #[default]
  Default,
  NodeQuery,
  Node {
    node: api::Node,
  },
  Package {
    handler: Hash,
    content: Hash,
  },
  PeerQuery {
    peer: Hash,
  },
  Peer {
    peer: Hash,
    packages: Option<BTreeMap<Hash, Manifest>>,
  },
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

  fn update(&mut self) -> impl DomFragment<Self> {
    use html::*;

    (
      nav((
        h1("System"),
        ul(li(button("Node").on_click(|state: &mut State, _| {
          state.view = View::NodeQuery;
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
      h2("Routing Table"),
      ol(
        node
          .routing_table
          .iter()
          .enumerate()
          .filter(|(_i, bucket)| !bucket.is_empty())
          .map(|(i, bucket)| {
            li(
              bucket
                .iter()
                .map(|peer| {
                  let peer = peer.id;
                  button(peer.to_string()).on_click(move |state: &mut State, _| {
                    state.view = View::PeerQuery { peer };
                  })
                })
                .collect::<Vec<_>>(),
            )
            .attr("value", i)
          })
          .collect::<Vec<_>>(),
      ),
      h2("Directory"),
      node
        .directory
        .iter()
        .map(|(hash, contacts)| {
          li((
            label(hash.to_string()),
            contacts
              .iter()
              .map(|contact| span(contact.to_string()))
              .collect::<Vec<_>>(),
          ))
        })
        .collect::<Vec<_>>(),
    ))
  }

  fn view(&self) -> impl DomView<State> {
    use html::*;

    match &self.view {
      View::Default => div(()).boxed(),
      View::NodeQuery => fork(
        div(()),
        memoized_await(
          (),
          |()| async { Api::default().node().await.unwrap() },
          |state: &mut State, node| state.view = View::Node { node },
        ),
      )
      .boxed(),
      View::Node { node } => self.node(node).boxed(),
      View::Package { handler, content } => iframe(())
        .attr("src", format!("/{handler}/{content}/"))
        .boxed(),
      View::PeerQuery { peer } => fork(
        div(()),
        memoized_await(
          *peer,
          |peer| {
            let peer = *peer;
            async move { (peer, Api::default().search(peer).await.unwrap()) }
          },
          |state: &mut State, output| {
            state.view = View::Peer {
              peer: output.0,
              packages: output.1,
            }
          },
        ),
      )
      .boxed(),
      View::Peer { peer, packages } => div((
        h1(peer.to_string()),
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

#[wasm_bindgen(main)]
fn main() -> Result<(), JsValue> {
  hypermedia::initialize_console(log::Level::Trace)?;
  App::new(
    web_sys::window()
      .unwrap()
      .document()
      .unwrap()
      .body()
      .unwrap(),
    State::default(),
    State::update,
  )
  .run();
  Ok(())
}
