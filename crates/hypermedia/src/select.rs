use super::*;

pub trait SelectDocumentFragment {
  fn select<T: JsCast>(&self, selector: &str) -> T {
    self
      .select_optional::<T>(selector)
      .expect("selector returned no elements")
  }

  fn select_optional<T: JsCast>(&self, selector: &str) -> Option<T>;

  fn select_some<T: JsCast>(&self, selector: &str) -> Vec<T> {
    let elements = self.select_all::<T>(selector);

    if elements.is_empty() {
      panic!("selector returned no elements");
    }

    elements
  }

  fn select_all<T: JsCast>(&self, selector: &str) -> Vec<T>;
}

impl<D: Deref<Target = DocumentFragment>> SelectDocumentFragment for D {
  fn select_optional<T: JsCast>(&self, selector: &str) -> Option<T> {
    self
      .query_selector(selector)
      .expect("invalid selector")
      .map(|element| element.cast::<T>())
  }

  fn select_all<T: JsCast>(&self, selector: &str) -> Vec<T> {
    let list = self.query_selector_all(selector).expect("invalid selector");
    let mut nodes = Vec::new();
    for i in 0..list.length() {
      let node = list.item(i).unwrap();
      nodes.push(node.cast::<T>());
    }
    nodes
  }
}

pub trait SelectElement {
  fn select<T: JsCast>(&self, selector: &str) -> T {
    self
      .select_optional::<T>(selector)
      .expect("selector returned no elements")
  }

  fn select_optional<T: JsCast>(&self, selector: &str) -> Option<T>;

  fn select_some<T: JsCast>(&self, selector: &str) -> Vec<T> {
    let elements = self.select_all::<T>(selector);

    if elements.is_empty() {
      panic!("selector returned no elements");
    }

    elements
  }

  fn select_all<T: JsCast>(&self, selector: &str) -> Vec<T>;
}

impl<D: Deref<Target = Element>> SelectElement for D {
  fn select_optional<T: JsCast>(&self, selector: &str) -> Option<T> {
    self
      .query_selector(selector)
      .expect("invalid selector")
      .map(|element| element.cast::<T>())
  }

  fn select_all<T: JsCast>(&self, selector: &str) -> Vec<T> {
    let list = self.query_selector_all(selector).expect("invalid selector");
    let mut nodes = Vec::new();
    for i in 0..list.length() {
      let node = list.item(i).unwrap();
      nodes.push(node.cast::<T>());
    }
    nodes
  }
}
