use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub))]
pub enum Error {
  #[snafu(display("failed to deserialize manifest"))]
  DeserializeManifest {
    backtrace: Backtrace,
    source: ciborium::de::Error<io::Error>,
  },
  #[snafu(display("package file hash `{hash}` duplicated"))]
  FileHashDuplicated { hash: Hash, backtrace: Backtrace },
  #[snafu(display("package file hash actually `{actual}` but expected `{expected}`"))]
  FileHashInvalid {
    actual: Hash,
    backtrace: Backtrace,
    expected: Hash,
  },
  #[snafu(display("package file hash `{hash}` out of order"))]
  FileHashOrder { hash: Hash, backtrace: Backtrace },
  #[snafu(display("package file length `{len}` cannot be converted to usize"))]
  FileLengthRange {
    backtrace: Backtrace,
    len: u64,
    source: TryFromIntError,
  },
  #[snafu(display("I/O error reading file `{path}`"))]
  FileIo {
    backtrace: Backtrace,
    path: Utf8PathBuf,
    source: io::Error,
  },
  #[snafu(transparent)]
  Io {
    backtrace: Backtrace,
    source: io::Error,
  },
  #[snafu(display("I/O error copying from `{path}`"))]
  IoCopy {
    backtrace: Backtrace,
    path: Utf8PathBuf,
    source: io::Error,
  },
  #[snafu(display(
    "unexpected package magic bytes {} (\"{}\")",
    hex::encode(magic),
    String::from_utf8_lossy(magic)
  ))]
  MagicBytes {
    backtrace: Backtrace,
    magic: [u8; 10],
  },
  #[snafu(display("manifest index {index} out of bounds of hash array"))]
  ManifestIndexOutOfBounds { backtrace: Backtrace, index: usize },
  #[snafu(display("could not convert manifest index {index} to usize"))]
  ManifestIndexRange {
    backtrace: Backtrace,
    index: u64,
    source: TryFromIntError,
  },
  #[snafu(display("package has trailing {trailing} bytes"))]
  TrailingBytes { backtrace: Backtrace, trailing: u64 },
}

#[derive(Debug)]
pub struct Package {
  pub files: HashMap<Hash, Vec<u8>>,
  pub manifest: Manifest,
}

impl Package {
  pub const MAGIC_BYTES: &'static str = "MEDIA📦\0";

  pub fn load(path: &Utf8Path) -> Result<Self, Error> {
    let file = File::open(path)?;

    let len = file.metadata()?.len();

    let mut package = BufReader::new(file);

    let mut magic = [0; Self::MAGIC_BYTES.len()];

    package.read_exact(&mut magic)?;

    ensure!(magic == Self::MAGIC_BYTES.as_bytes(), MagicBytes { magic });

    let index = package.read_u64()?;

    let index = usize::try_from(index).context(ManifestIndexRange { index })?;

    let hash_count = package.read_u64()?;

    let mut hashes = Vec::new();

    for _ in 0..hash_count {
      let hash = package.read_hash()?;
      let len = package.read_u64()?;

      usize::try_from(len).context(FileLengthRange { len })?;

      hashes.push((hash, len));
    }

    let manifest = hashes
      .get(index)
      .context(ManifestIndexOutOfBounds { index })?
      .0;

    let mut files = HashMap::<Hash, Vec<u8>>::new();

    let mut last = Option::<Hash>::None;

    for (expected, len) in hashes {
      if let Some(last) = last {
        ensure!(
          expected.as_bytes() >= last.as_bytes(),
          FileHashOrder { hash: expected }
        );

        ensure!(
          expected.as_bytes() != last.as_bytes(),
          FileHashDuplicated { hash: expected }
        );
      }

      last = Some(expected);

      let mut buffer = vec![0; len as usize];

      package.read_exact(&mut buffer)?;

      let actual = blake3::hash(&buffer);

      ensure!(actual == expected, FileHashInvalid { expected, actual });

      files.insert(expected, buffer);
    }

    let position = package.stream_position()?;

    ensure!(
      position == len,
      TrailingBytes {
        trailing: len.saturating_sub(position),
      }
    );

    let manifest = ciborium::from_reader(Cursor::new(files.get(&manifest).unwrap()))
      .context(DeserializeManifest)?;

    Ok(Self { manifest, files })
  }

  pub fn save(
    root: &Utf8Path,
    output: &Utf8Path,
    hashes: HashMap<Utf8PathBuf, (Hash, u64)>,
    template: Template,
  ) -> Result<(), Error> {
    let mut package = BufWriter::new(File::create(&output)?);

    package.write_all(crate::package::Package::MAGIC_BYTES.as_bytes())?;

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

    package.write_u64(manifest_index)?;

    package.write_u64(hashes_sorted.len().into_u64())?;

    for (hash, len) in &hashes_sorted {
      package.write_hash(*hash)?;
      package.write_u64(*len)?;
    }

    let paths = hashes
      .into_iter()
      .map(|(path, (hash, _len))| (hash, path))
      .collect::<HashMap<Hash, Utf8PathBuf>>();

    for (hash, _len) in hashes_sorted {
      if hash == manifest_hash {
        package.write_all(&manifest)?;
      } else {
        let path = root.join(paths.get(&hash).unwrap());

        let mut file = File::open(&path).context(FileIo { path: &path })?;

        io::copy(&mut file, &mut package).context(IoCopy { path: &path })?;
      }
    }

    Ok(())
  }

  pub fn file(&self, path: &str) -> Option<(Mime, Vec<u8>)> {
    match &self.manifest {
      Manifest::App { paths, .. } => {
        let hash = paths.get(path)?;

        Some((
          mime_guess::from_path(path).first_or_octet_stream(),
          self.files.get(hash).unwrap().clone(),
        ))
      }
      Manifest::Comic { pages } => Some((
        mime::IMAGE_JPEG,
        self
          .files
          .get(pages.get(path.parse::<usize>().ok()?)?)
          .unwrap()
          .clone(),
      )),
    }
  }
}
