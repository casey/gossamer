use super::*;

pub trait Cast {
  fn cast<T: JsCast>(self) -> T;
}

impl<V: JsCast + std::fmt::Debug> Cast for V {
  fn cast<T: JsCast>(self) -> T {
    self.dyn_into::<T>().expect("cast failed")
  }
}
