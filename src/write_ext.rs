use super::*;

pub(crate) trait WriteExt {
  fn write_hash(&mut self, value: Hash) -> io::Result<()>;

  fn write_u64(&mut self, value: u64) -> io::Result<()>;
}

impl<T: Write> WriteExt for T {
  fn write_hash(&mut self, value: Hash) -> io::Result<()> {
    self.write_all(value.as_bytes())
  }

  fn write_u64(&mut self, value: u64) -> io::Result<()> {
    self.write_all(&value.to_le_bytes())
  }
}
