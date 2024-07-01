use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub))]
pub enum Error {
  #[snafu(display("failed to deserialize manifest"))]
  DeserializeManifest {
    backtrace: Backtrace,
    source: ciborium::de::Error<io::Error>,
  },
  #[snafu(display("I/O error reading package"), context(false))]
  Io {
    backtrace: Backtrace,
    source: io::Error,
  },
  #[snafu(display("package file hash `{hash}` duplicated"))]
  FileHashDuplicated { hash: Hash, backtrace: Backtrace },
  #[snafu(display("package file length `{len}` cannot be converted to usize"))]
  FileLengthRange {
    backtrace: Backtrace,
    len: u64,
    source: TryFromIntError,
  },
  #[snafu(display("package file hash `{hash}` out of order"))]
  FileHashOrder { hash: Hash, backtrace: Backtrace },
  #[snafu(display("package file hash actually `{actual}` but expected `{expected}`"))]
  FileHashInvalid {
    actual: Hash,
    backtrace: Backtrace,
    expected: Hash,
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
  #[snafu(display("package has trailing {trailing} bytes"))]
  TrailingBytes { backtrace: Backtrace, trailing: u64 },
  #[snafu(display("manifest index {index} out of bounds of hash array"))]
  ManifestIndexOutOfBounds { backtrace: Backtrace, index: usize },
  #[snafu(display("could not convert manifest index {index} to usize"))]
  ManifestIndexRange {
    backtrace: Backtrace,
    index: u64,
    source: TryFromIntError,
  },
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
