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
    if self.output.starts_with(&self.root) {
      return Err(Error::OutputInRoot {
        backtrace: Backtrace::capture(),
        output: self.output,
        root: self.root,
      });
    }

    let metadata = Metadata::load(&self.root.join(Metadata::PATH))?;

    let paths = self.paths()?;

    let template = metadata.template(&self.root, &paths)?;

    let hashes = self.hashes(paths)?;

    self.write(hashes, template)?;

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

    if !paths.contains(Utf8Path::new(Metadata::PATH)) {
      return Err(Error::MetadataMissing {
        backtrace: Backtrace::capture(),
        root: self.root.clone(),
      });
    }

    Ok(paths)
  }

  fn write(&self, hashes: HashMap<Utf8PathBuf, (Hash, u64)>, template: Template) -> Result {
    let context = error::Io { path: &self.output };

    let mut package = BufWriter::new(File::create(&self.output).context(context)?);

    package
      .write_all(crate::package::Package::MAGIC_BYTES.as_bytes())
      .context(context)?;

    let mut hashes_sorted = hashes.values().copied().collect::<Vec<(Hash, u64)>>();

    let manifest = {
      let mut buffer = Vec::new();
      ciborium::into_writer(&template.manifest(&hashes), &mut buffer).unwrap();
      buffer
    };

    let manifest_hash = blake3::hash(&manifest);

    hashes_sorted.push((manifest_hash, manifest.len().into_u64()));

    hashes_sorted.sort_by_key(|hash| *hash.0.as_bytes());

    let manifest_index = hashes_sorted
      .iter()
      .position(|(hash, _len)| *hash == manifest_hash)
      .unwrap()
      .into_u64();

    package.write_u64(manifest_index).context(context)?;

    package
      .write_u64(hashes_sorted.len().into_u64())
      .context(context)?;

    for (hash, len) in &hashes_sorted {
      package.write_hash(*hash).context(context)?;
      package.write_u64(*len).context(context)?;
    }

    let paths = hashes
      .into_iter()
      .map(|(path, (hash, _len))| (hash, path))
      .collect::<HashMap<Hash, Utf8PathBuf>>();

    for (hash, _len) in hashes_sorted {
      if hash == manifest_hash {
        package.write_all(&manifest).context(context)?;
      } else {
        let path = self.root.join(paths.get(&hash).unwrap());

        let mut file = File::open(&path).context(context)?;

        io::copy(&mut file, &mut package).context(error::IoCopy {
          from: &path,
          to: &self.output,
        })?;
      }
    }

    Ok(())
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
        eprintln!("error packaging {root}: {err}");

        for (i, err) in err.iter_chain().skip(1).enumerate() {
          if i == 0 {
            eprintln!();
            eprintln!("because:");
          }

          eprintln!("- {err}");
        }

        if let Some(backtrace) = err.backtrace() {
          if backtrace.status() == BacktraceStatus::Captured {
            eprintln!("backtrace:");
            eprintln!("{backtrace}");
          }
        }

        panic!("packaging {root} failed");
      }
    }
  }
}
