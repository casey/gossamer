#[allow(unused)]
macro_rules! debug {
  () => {
    log::debug!("[{}:{}]", file!(), line!());
  };
  ($val:expr) => {
    match $val {
      tmp => {
        log::debug!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($val), &tmp);
        tmp
      }
    }
  };
  ($val:expr,) => { debug!($val) };
  ($($val:expr),+ $(,)?) => {
    ($(debug!($val)),+,)
  };
}
