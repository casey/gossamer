use super::*;

#[derive(Debug)]
pub struct Package {
  pub files: HashMap<Hash, Vec<u8>>,
  pub manifest: Manifest,
}

impl Package {
  pub fn load(path: &Utf8Path) -> Result<Self> {
    let context = error::Io { path };

    let file = File::open(path).context(context)?;

    let len = file.metadata().context(context)?.len();

    let mut package = BufReader::new(file);

    let manifest_index = usize::try_from(package.read_u64().context(context)?)
      .context(error::ManifestIndexRange { package: &path })?;

    let hash_count = package.read_u64().context(context)?;

    let mut hashes = Vec::new();

    for _ in 0..hash_count {
      let hash = package.read_hash().context(context)?;
      let len = package.read_u64().context(context)?;

      hashes.push((hash, len));
    }

    let manifest = hashes
      .get(manifest_index)
      .context(error::ManifestIndexOutOfBounds { package: &path })?
      .0;

    let mut files = HashMap::<Hash, Vec<u8>>::new();

    for (expected, len) in hashes {
      let mut buffer = vec![0; len as usize];

      package.read_exact(&mut buffer).context(context)?;

      let actual = blake3::hash(&buffer);

      if actual != expected {
        return Err(Error::PackageFileHash {
          backtrace: Backtrace::capture(),
          expected,
          actual,
        });
      }

      files.insert(expected, buffer);
    }

    let position = package.stream_position().context(context)?;

    if position != len {
      return Err(Error::PackageTrailingBytes {
        backtrace: Backtrace::capture(),
        trailing: len.saturating_sub(position),
      });
    }

    let manifest = ciborium::from_reader(Cursor::new(files.get(&manifest).unwrap()))
      .context(error::DeserializeManifest { path })?;

    Ok(Self { manifest, files })
  }

  pub fn get(&self, path: &str) -> Option<(String, Vec<u8>)> {
    match &self.manifest {
      Manifest::App { paths, .. } => {
        let hash = paths.get(path)?;

        Some((
          mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string(),
          self.files.get(hash).unwrap().clone(),
        ))
      }
      Manifest::Comic { pages } => Some((
        "image/jpeg".into(),
        self
          .files
          .get(pages.get(path.parse::<usize>().ok()?)?)
          .unwrap()
          .clone(),
      )),
    }
  }
}
