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
    media::{Hash, Manifest, Target, Type},
    wasm_bindgen::{self, prelude::wasm_bindgen, JsValue},
    Api,
  },
  std::collections::BTreeMap,
  xilem_web::{
    concurrent::await_once,
    core::fork,
    document_body,
    elements::html,
    interfaces::{Element, HtmlButtonElement, HtmlDivElement, HtmlElement, HtmlLiElement},
    AnyDomView, App, DomView,
  },
};

#[derive(Debug)]
struct State {
  handlers: BTreeMap<Target, Hash>,
  packages: Vec<(Hash, Manifest)>,
  package: Option<(Hash, Hash)>,
  origin: String,
}

impl State {
  fn new() -> Self {
    Self {
      handlers: BTreeMap::new(),
      packages: Vec::new(),
      package: None,
      origin: web_sys::window().unwrap().location().origin().unwrap(),
    }
  }

  fn update(self: &mut State) -> impl DomView<State> {
    use html::*;

    log::debug!("{:?}", self);

    fork(
      div((
        nav((
          h1("System"),
          h1("Network"),
          h1("Apps"),
          ul(
            self
              .packages
              .iter()
              .filter(|(_hash, manifest)| manifest.ty() == Type::App)
              .map(|(content, manifest)| {
                li(
                  if let Some(handler) = self.handlers.get(&manifest.ty().into()) {
                    log::debug!("{handler}");
                    let handler = *handler;
                    let content = *content;
                    button(manifest.name.clone())
                      .on_click(move |state: &mut State, _| {
                        state.package = Some((handler, content));
                      })
                      .boxed()
                  } else {
                    manifest.name.clone().boxed()
                  },
                )
              })
              .collect::<Vec<_>>(),
          ),
          h1("Content"),
          ul(
            self
              .packages
              .iter()
              .filter(|(_hash, manifest)| manifest.ty() != Type::App)
              .map(|(content, manifest)| {
                li(
                  if let Some(handler) = self.handlers.get(&manifest.ty().into()) {
                    log::debug!("{handler}");
                    let handler = *handler;
                    let content = *content;
                    button(manifest.name.clone())
                      .on_click(move |state: &mut State, _| {
                        state.package = Some((handler, content));
                      })
                      .boxed()
                  } else {
                    manifest.name.clone().boxed()
                  },
                )
              })
              .collect::<Vec<_>>(),
          ),
        )),
        main((
          iframe(()).attr(
            "src",
            self
              .package
              .map(|(handler, content)| format!("{}/{handler}/{content}", self.origin)),
          ),
          section(()),
        )),
      )),
      (
        await_once(
          |_| async {
            Api::default()
              .packages()
              .await
              .unwrap()
              .into_iter()
              .collect::<Vec<(Hash, Manifest)>>()
          },
          |state: &mut State, output| state.packages = output,
        ),
        await_once(
          |_| async { Api::default().handlers().await.unwrap() },
          |state: &mut State, output| state.handlers = output,
        ),
      ),
    )
  }
}

//     <nav>
//       <h1>System</h1>
//       <ul>
//         <li><button id=node>Node</button></li>
//       </ul>
//       <h1>Network</h1>
//       <ul>
//         <li><button id=search>Search</button></li>
//       </ul>
//       <h1>Apps</h1>
//       <ul>
// %% for (hash, manifest) in self.packages.iter().filter(|(hash, manifest)| manifest.ty() == Type::App) {
// %%   let ty = manifest.ty();
//         <li>
// %%   if let Some(handler) = self.handlers.get(&ty.into()) {
//           <button class=package data-handler={{handler}} data-package={{hash}}>{{ manifest.name }}</button>
// %%   } else {
//           {{ manifest.name }}
// %%   }
//         </li>
// %% }
//       </ul>
//       <h1>Content</h1>
//       <ul>
// %% for (hash, manifest) in self.packages.iter().filter(|(hash, manifest)| manifest.ty() != Type::App) {
// %%   let ty = manifest.ty();
//         <li>
// %%   if let Some(handler) = self.handlers.get(&ty.into()) {
//           <button class=package data-handler={{handler}} data-package={{hash}}>{{ manifest.name }}</button>
//   %%   } else {
//           {{ manifest.name }}
// %%   }
//         </li>
// %% }
//       </ul>
//     </nav>
//     <main>
//       <iframe></iframe>
//       <section></section>
//     </main>
//

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
