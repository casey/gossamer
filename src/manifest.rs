use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Manifest {
  App {
    handles: Type,
    paths: BTreeMap<String, Hash>,
  },
  Comic {
    pages: Vec<Hash>,
  },
}

impl Manifest {
  pub fn ty(&self) -> Type {
    match self {
      Self::App { .. } => Type::App,
      Self::Comic { .. } => Type::Comic,
    }
  }

  pub fn verify(
    &self,
    manifest: Hash,
    files: &HashMap<Hash, Vec<u8>>,
  ) -> Result<(), package::Error> {
    let mut extra = 0u64;
    let mut missing = 0u64;

    let expected: HashSet<Hash> = match self {
      Self::App { paths, .. } => paths.values().copied().collect(),
      Self::Comic { pages } => pages.iter().copied().collect(),
    };

    for hash in &expected {
      if !files.contains_key(hash) {
        missing += 1;
      }
    }

    ensure!(missing == 0, package::ManifestMissingFiles { missing });

    for hash in files.keys() {
      if *hash != manifest && !expected.contains(hash) {
        extra += 1;
      }
    }

    ensure!(extra == 0, package::ManifestExtraFiles { extra });

    Ok(())
  }
}
