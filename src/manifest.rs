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

  pub fn verify(&self, files: &HashMap<Hash, Vec<u8>>) -> Result<(), package::Error> {
    let mut extra = 0u64;
    let mut missing = 0u64;

    match self {
      Self::App { paths, .. } => {
        for (_, hash) in paths {
          if !files.contains_key(hash) {
            missing += 1;
          }
        }
      }
      Self::Comic { pages } => {
        for hash in pages {
          if !files.contains_key(hash) {
            missing += 1;
          }
        }

        let pages = pages.iter().copied().collect::<HashSet<Hash>>();

        for (hash, _) in files {
          if !pages.contains(hash) {
            extra += 1;
          }
        }
      }
    }

    ensure!(missing == 0, package::ManifestMissingFiles { missing });
    ensure!(extra == 0, package::ManifestExtraFiles { extra });

    Ok(())
  }
}
