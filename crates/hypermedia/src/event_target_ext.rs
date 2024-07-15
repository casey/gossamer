use super::*;

pub trait EventTargetExt {
  fn add_event_listener<E, F, R>(&self, event_type: &str, callback: F)
  where
    E: FromWasmAbi + 'static,
    F: FnMut(E) -> R + 'static,
    R: IntoWasmAbi + 'static;
}

impl<T: Deref<Target = EventTarget>> EventTargetExt for T {
  fn add_event_listener<E, F, R>(&self, event_type: &str, callback: F)
  where
    E: FromWasmAbi + 'static,
    F: FnMut(E) -> R + 'static,
    R: IntoWasmAbi + 'static,
  {
    let closure = Closure::new(callback);
    self
      .add_event_listener_with_callback(event_type, closure.as_ref().dyn_ref().unwrap())
      .unwrap();
    closure.forget();
  }
}
