use super::*;

pub(crate) trait Report {
  fn report(&self);
}

impl<T: Display + ErrorCompat + std::error::Error + 'static> Report for T {
  fn report(&self) {
    eprintln!("error: {self}");

    for (i, err) in self.iter_chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("because:");
      }

      eprintln!("- {err}");
    }

    if let Some(backtrace) = self.backtrace() {
      if backtrace.status() == BacktraceStatus::Captured {
        eprintln!();
        eprintln!("backtrace:");
        eprintln!("{backtrace}");
      }
    }
  }
}
