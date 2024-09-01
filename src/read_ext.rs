use super::*;

pub(crate) trait ReadExt {
  fn read_hash(&mut self) -> io::Result<Hash>;

  fn read_u64(&mut self) -> io::Result<u64>;
}

impl<T: Read> ReadExt for T {
  fn read_hash(&mut self) -> io::Result<Hash> {
    let mut array = [0u8; 32];

    self.read_exact(&mut array)?;

    Ok(array.into())
  }

  fn read_u64(&mut self) -> io::Result<u64> {
    let mut array = [0u8; 8];

    self.read_exact(&mut array)?;

    Ok(u64::from_le_bytes(array))
  }
}
