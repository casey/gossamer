use super::*;

pub trait EventTargetExt {
  fn add_event_listener<E: FromWasmAbi + 'static, F: FnMut(E) + 'static>(
    &self,
    event_type: &str,
    callback: F,
  );
}

impl<T: Deref<Target = EventTarget>> EventTargetExt for T {
  fn add_event_listener<E: FromWasmAbi + 'static, F: FnMut(E) + 'static>(
    &self,
    event_type: &str,
    callback: F,
  ) {
    let closure = Closure::new(callback);
    self
      .add_event_listener_with_callback(event_type, closure.as_ref().dyn_ref().unwrap())
      .unwrap();
    closure.forget();
  }
}
