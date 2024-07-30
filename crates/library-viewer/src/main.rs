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
    media::{api, Hash, Manifest, Peer, Target, Type},
    wasm_bindgen::{self, prelude::wasm_bindgen, JsValue},
    Api, Cast,
  },
  std::collections::BTreeMap,
  xilem_web::{
    concurrent::memoized_await, core::fork, elements::html, interfaces::Element, App, DomView,
  },
};

#[derive(Debug)]
struct State {
  handlers: BTreeMap<Target, Hash>,
  node: Option<api::Node>,
  packages: Vec<(Hash, Manifest)>,
  query: Option<Hash>,
  selection: Option<Selection>,
  results: Option<Option<Peer>>,
}

#[derive(Debug, PartialEq)]
enum Selection {
  Node,
  Package(Hash, Hash),
  Search,
}

impl State {
  fn new() -> Self {
    Self {
      handlers: BTreeMap::new(),
      node: None,
      packages: Vec::new(),
      query: None,
      selection: None,
      results: None,
    }
  }

  fn package_list(self: &mut State, apps: bool) -> impl DomView<Self> {
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
                  state.selection = Some(Selection::Package(handler, content));
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

  fn update(self: &mut State) -> impl DomView<Self> {
    use html::*;

    fork(
      div((
        nav((
          h1("System"),
          button("Search").on_click(|state: &mut State, _| {
            state.selection = Some(Selection::Search);
          }),
          h1("Network"),
          button("Node").on_click(|state: &mut State, _| {
            state.selection = Some(Selection::Node);
          }),
          h1("Apps"),
          self.package_list(true),
          h1("Content"),
          self.package_list(false),
        )),
        main(self.selection.as_ref().map(|selection| {
          match selection {
            Selection::Node => self.node.as_ref().map(|node| self.node(node).boxed()),
            Selection::Package(handler, content) => Some(
              iframe(())
                .attr("src", format!("/{handler}/{content}/"))
                .boxed(),
            ),
            Selection::Search => Some(
              div((
                h1("Search"),
                input(())
                  .attr("type", "text")
                  .on_input(|state: &mut State, event| {
                    state.query = event
                      .target()
                      .unwrap()
                      .cast::<web_sys::HtmlInputElement>()
                      .value()
                      .parse()
                      .ok();
                  }),
                self.results.as_ref().map(|peer| {
                  div(if let Some(peer) = peer {
                    peer.to_string()
                  } else {
                    "hash not found".to_string()
                  })
                }),
              ))
              .boxed(),
            ),
          }
        })),
      )),
      (
        memoized_await(
          (),
          |()| async {
            Api::default()
              .packages()
              .await
              .unwrap()
              .into_iter()
              .collect::<Vec<(Hash, Manifest)>>()
          },
          |state: &mut State, output| state.packages = output,
        ),
        memoized_await(
          (),
          |()| async { Api::default().handlers().await.unwrap() },
          |state: &mut State, output| state.handlers = output,
        ),
        memoized_await(
          self.selection == Some(Selection::Node),
          |node| {
            let node = *node;
            async move {
              if node {
                Some(Api::default().node().await.unwrap())
              } else {
                None
              }
            }
          },
          |state: &mut State, output| state.node = output,
        ),
        memoized_await(
          self.query,
          |query| {
            let query = *query;
            async move {
              if let Some(query) = query {
                Some(Api::default().search(query).await.unwrap())
              } else {
                None
              }
            }
          },
          |state: &mut State, output| state.results = output,
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
                .map(|peer| span(peer.to_string()))
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
    State::new(),
    State::update,
  )
  .run();
  Ok(())
}
