use super::*;

// show an error if it's the wrong kind of manifest
//
// i feel like i shouldn't just commit content
// - generate content, works okay for jpegs
// - commit content: works as long as they're small

// gallery is:
// - title
// - isbn
// - series
// - volume
// - part
// - artists
// - publisher
// - original language
// - translation language
// - source: digital or scan
//
// - files must all be same format
// - files must all be same size, double pages allowed
// - files must be recognized format: jpg
//
// manga/comic
// movie
// videos
// tv show
// music
// console games
//
// music
// videos
// movies / tv shows
//
// // check that they're actually JPGs
// // - check magic number
// // - deserialize

#[derive(Parser)]
pub struct Package {
  #[arg(long, help = "Package contents of directory <ROOT>.")]
  root: Utf8PathBuf,
  #[arg(long, help = "Save package to <OUTPUT>.")]
  output: Utf8PathBuf,
}

impl Package {
  pub fn run(self) -> Result {
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
        root: self.root,
      });
    }

    let template = {
      let path = self.root.join(Metadata::PATH);

      let file = File::open(&path).context(error::Io { path: &path })?;

      let metadata: Metadata =
        serde_yaml::from_reader(&file).context(error::DeserializeMetadata { path: &path })?;

      match metadata.ty {
        Type::App => {
          if !paths.contains(Utf8Path::new("index.html")) {
            return Err(Error::Index {
              backtrace: Backtrace::capture(),
              root: self.root,
            });
          };
          Template::App
        }
        Type::Comic => {
          let mut pages: Vec<(u64, Utf8PathBuf)> = Vec::new();

          let page_re = Regex::new(r"^(\d+)\.jpg$").unwrap();

          for path in &paths {
            if path == Metadata::PATH {
              continue;
            }

            let captures = page_re
              .captures(path.as_ref())
              .context(error::UnexpectedFile {
                file: path.clone(),
                ty: metadata.ty,
              })?;

            pages.push((
              captures[1].parse().context(error::InvalidPage { path })?,
              path.clone(),
            ));
          }

          pages.sort();

          for (i, (page, _path)) in pages.iter().enumerate() {
            let i = i.into_u64();
            let page = *page;

            if i < page {
              return Err(Error::PageMissing {
                backtrace: Backtrace::capture(),
                page: i,
              });
            }

            if i > page {
              return Err(Error::PageDuplicated {
                backtrace: Backtrace::capture(),
                page,
              });
            }
          }

          Template::Comic {
            pages: pages.into_iter().map(|(_page, path)| path).collect(),
          }
        }
      }
    };

    let mut hashes = HashMap::new();

    for p in paths {
      let path = self.root.join(&p);

      let file = File::open(&path).context(error::Io { path: &path })?;

      let len = file.metadata().context(error::Io { path: &path })?.len();

      let mut hasher = Hasher::new();

      hasher
        .update_reader(file)
        .context(error::Io { path: &path })?;

      let hash = hasher.finalize();

      hashes.insert(p, (hash, len));
    }

    let mut package =
      BufWriter::new(File::create(&self.output).context(error::Io { path: &self.output })?);

    let mut hashes_sorted = hashes
      .iter()
      .map(|(_path, hash)| *hash)
      .collect::<Vec<(Hash, u64)>>();

    let manifest = {
      let manifest = match template {
        Template::App => {
          let mut paths = BTreeMap::new();

          for (path, (hash, _len)) in &hashes {
            paths.insert(path.to_string(), *hash);
          }

          Manifest::App { paths }
        }
        Template::Comic { pages } => Manifest::Comic {
          pages: pages
            .into_iter()
            .map(|path| hashes.get(&path).unwrap().0)
            .collect(),
        },
      };

      let mut buffer = Vec::new();
      ciborium::into_writer(&manifest, &mut buffer).unwrap();

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

    package
      .write_u64(manifest_index)
      .context(error::Io { path: &self.output })?;

    package
      .write_u64(hashes_sorted.len().into_u64())
      .context(error::Io { path: &self.output })?;

    for (hash, len) in &hashes_sorted {
      package
        .write_hash(*hash)
        .context(error::Io { path: &self.output })?;
      package
        .write_u64(*len)
        .context(error::Io { path: &self.output })?;
    }

    let paths = hashes
      .into_iter()
      .map(|(path, (hash, _len))| (hash, path))
      .collect::<HashMap<Hash, Utf8PathBuf>>();

    for (hash, _len) in hashes_sorted {
      if hash == manifest_hash {
        package
          .write_all(&manifest)
          .context(error::Io { path: &self.output })?;
      } else {
        let path = self.root.join(paths.get(&hash).unwrap());

        let mut file = File::open(&path).context(error::Io { path: &path })?;

        // todo: check that we're not copying from package
        io::copy(&mut file, &mut package).context(error::IoCopy {
          from: &path,
          to: &self.output,
        })?;
      }
    }

    Ok(())
  }
}
