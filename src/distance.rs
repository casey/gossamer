use super::*;

#[derive(Default, Eq, PartialEq)]
pub(crate) struct Distance([u8; 32]);

impl Distance {
  pub(crate) fn bucket(self) -> usize {
    self
      .0
      .iter()
      .enumerate()
      .rev()
      .find(|(_i, value)| **value != 0)
      .map(|(i, value)| i * 8 + 8 - usize::try_from(value.leading_zeros()).unwrap())
      .unwrap_or_default()
  }

  pub(crate) fn new(a: Hash, b: Hash) -> Self {
    let mut bytes = [0; 32];

    for (d, (a, b)) in bytes.iter_mut().zip(a.as_bytes().iter().zip(b.as_bytes())) {
      *d = a ^ b;
    }

    Self(bytes)
  }
}

impl Ord for Distance {
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .0
      .iter()
      .rev()
      .zip(other.0.iter().rev())
      .map(|(s, o)| s.cmp(o))
      .find(|o| *o != Ordering::Equal)
      .unwrap_or(Ordering::Equal)
  }
}

impl PartialOrd for Distance {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn distance(i: u32) -> Distance {
    let mut low = 0u128;
    let mut high = 0u128;

    if i < 128 {
      low = 1 << i;
    } else {
      high = 1 << (i - 128);
    };

    let mut bytes = [0; 32];

    for (i, b) in low
      .to_le_bytes()
      .iter()
      .chain(high.to_le_bytes().iter())
      .enumerate()
    {
      bytes[i] = *b;
    }

    Distance(bytes)
  }

  #[test]
  fn distance_bucket() {
    #[track_caller]
    fn case(i: u32, bucket: usize) {
      assert_eq!(distance(i).bucket(), bucket)
    }

    let hash = Hash::bytes(&[]);
    assert_eq!(Distance::new(hash, hash).bucket(), 0);

    case(0, 1);
    case(1, 2);
    case(2, 3);
    case(254, 255);
    case(255, 256);
  }

  #[test]
  fn distance_ord() {
    assert!(distance(0) < distance(1));
    assert!(distance(0) < distance(8));
    assert!(distance(7) < distance(8));
    assert!(distance(8) < distance(9));
  }
}
