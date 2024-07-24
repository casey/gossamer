use super::*;

pub(crate) struct DeserializeFromStr<T: FromStr>(pub(crate) T);

impl<'de, T: FromStr> Deserialize<'de> for DeserializeFromStr<T>
where
  T::Err: Display,
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(Self(
      FromStr::from_str(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)?,
    ))
  }
}

impl<T: FromStr> Deref for DeserializeFromStr<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T: FromStr> DerefMut for DeserializeFromStr<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn deserialize_from_str() {
    assert_eq!(
      serde_json::from_str::<DeserializeFromStr<u64>>("\"1\"")
        .unwrap()
        .0,
      1,
    );
  }
}
