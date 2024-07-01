use super::*;

#[derive(Parser)]
pub struct Package {
  #[arg(long, help = "Package contents of directory <ROOT>.")]
  root: Utf8PathBuf,
  #[arg(long, help = "Save package to <OUTPUT>.")]
  output: Utf8PathBuf,
}

impl Package {
  pub fn run(self) -> Result {
    ensure!(
      !self.output.starts_with(&self.root),
      error::OutputInRoot {
        output: self.output,
        root: self.root,
      }
    );

    let metadata = Metadata::load(&self.root.join(Metadata::PATH))?;

    let paths = self.paths()?;

    let template = metadata.template(&self.root, &paths)?;

    let hashes = self.hashes(paths)?;

    let manifest = template.manifest(&hashes);

    crate::package::Package::save(hashes, manifest, &self.output, &self.root)
      .context(error::PackageSave { path: &self.output })?;

    Ok(())
  }

  fn hashes(&self, paths: HashSet<Utf8PathBuf>) -> Result<HashMap<Utf8PathBuf, (Hash, u64)>> {
    let mut hashes = HashMap::new();

    for relative in paths {
      let path = self.root.join(&relative);

      let context = error::Io { path: &path };

      let file = File::open(&path).context(context)?;

      let len = file.metadata().context(context)?.len();

      let mut hasher = Hasher::new();

      hasher.update_reader(file).context(context)?;

      let hash = hasher.finalize();

      hashes.insert(relative.clone(), (hash, len));
    }

    Ok(hashes)
  }

  fn paths(&self) -> Result<HashSet<Utf8PathBuf>> {
    let mut paths = HashSet::new();

    for result in WalkDir::new(&self.root) {
      let entry = result.context(error::WalkDir { root: &self.root })?;

      if entry.file_type().is_dir() || entry.file_name() == ".DS_Store" {
        continue;
      }

      paths.insert(
        entry
          .path()
          .try_into_utf8()?
          .strip_prefix(&self.root)
          .unwrap()
          .to_owned(),
      );
    }

    ensure!(
      paths.contains(Utf8Path::new(Metadata::PATH)),
      error::MetadataMissing {
        root: self.root.clone(),
      }
    );

    Ok(paths)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn package() {
    for root in ["apps/comic", "content/comic"] {
      let tempdir = tempfile::tempdir().unwrap();

      let result = Package {
        root: root.into(),
        output: Utf8Path::from_path(tempdir.path())
          .unwrap()
          .join("output.package"),
      }
      .run();

      if let Err(err) = result {
        err.report();
        panic!("packaging {root} failed");
      }
    }
  }
}
