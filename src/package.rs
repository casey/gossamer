use super::*;

#[derive(Debug)]
pub struct Package {
  pub files: HashMap<Hash, Vec<u8>>,
  pub manifest: Manifest,
}

impl Package {
  pub fn load(path: &Utf8Path) -> Result<Self> {
    let context = error::Io { path };

    let mut package = BufReader::new(File::open(path).context(context)?);

    let manifest_index = package.read_u64().context(context)?;

    let hash_count = package.read_u64().context(context)?;

    let mut hashes = Vec::new();

    let mut manifest = None;

    for i in 0..hash_count {
      let hash = package.read_hash().context(context)?;
      let len = package.read_u64().context(context)?;

      hashes.push((hash, len));

      if i == manifest_index {
        manifest = Some(hash);
      }
    }

    let mut files = HashMap::<Hash, Vec<u8>>::new();

    for (hash, len) in hashes {
      let mut buffer = vec![0; len as usize];

      package.read_exact(&mut buffer).context(context)?;

      files.insert(hash, buffer);
    }

    let manifest = ciborium::from_reader(Cursor::new(files.get(&manifest.unwrap()).unwrap()))
      .context(error::DeserializeManifest { path })?;

    Ok(Self { manifest, files })
  }

  pub fn get(&self, path: &str) -> Option<(String, Vec<u8>)> {
    match &self.manifest {
      Manifest::App { paths, .. } => {
        let Some(hash) = paths.get(path) else {
          return None;
        };

        Some((
          mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string(),
          self.files.get(hash).unwrap().clone(),
        ))
      }
      Manifest::Comic { pages } => {
        if path == "index.html" {
          let mut index = String::new();

          for (i, _hash) in pages.iter().enumerate() {
            index.push_str(&format!("<img src={i}>\n"));
          }

          Some(("text/html".into(), index.into_bytes()))
        } else if let Ok(page) = path.parse::<usize>() {
          Some((
            "image/jpeg".into(),
            self.files.get(&pages[page]).unwrap().clone(),
          ))
        } else {
          return None;
        }
      }
    }
  }
}
