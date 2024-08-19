pub(crate) trait IntoU64 {
  fn into_u64(self) -> u64;
}

impl IntoU64 for usize {
  fn into_u64(self) -> u64 {
    self.try_into().unwrap()
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn into_u64() {
    assert!(usize::MAX as u128 <= u64::MAX as u128);
  }
}
