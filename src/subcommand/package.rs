use super::*;

#[derive(Parser)]
pub struct Package {
  #[arg(long, help = "Package contents of directory <ROOT>.")]
  root: Utf8PathBuf,
  #[arg(long, help = "Save package to <OUTPUT>.")]
  output: Utf8PathBuf,
}

// todo:
//
// package command:
// - comic packaging:
//   - UnexpectedFile on non-page
//   - InvalidPage page number too large for u64
//   - PageMissing
//   - PageDuplicated
//   - leading zeros not allowed?
//
// - serve command
// - package save
// - package load

impl Package {
  pub fn run(self) -> Result {
    ensure!(
      !self.output.starts_with(&self.root),
      error::OutputInRoot {
        output: self.output,
        root: self.root,
      }
    );

    ensure!(
      !self.output.is_dir(),
      error::OutputIsDir {
        output: self.output
      },
    );

    let metadata = self.root.join(Metadata::PATH);

    ensure!(
      metadata.exists(),
      error::MetadataMissing { root: &self.root },
    );

    let metadata = Metadata::load(&metadata)?;

    let paths = self.paths()?;

    let template = metadata.template(&self.root, &paths)?;

    let hashes = self.hashes(paths)?;

    let manifest = template.manifest(&hashes);

    super::Package::save(hashes, manifest, &self.output, &self.root)
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

      let path = entry
        .path()
        .try_into_utf8()?
        .strip_prefix(&self.root)
        .unwrap()
        .to_owned();

      if path == Utf8Path::new(Metadata::PATH) {
        continue;
      }

      paths.insert(path);
    }

    Ok(paths)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
  }

  trait TempDirExt {
    fn path_utf8(&self) -> &Utf8Path;
  }

  impl TempDirExt for tempfile::TempDir {
    fn path_utf8(&self) -> &Utf8Path {
      self.path().try_into().unwrap()
    }
  }

  #[test]
  fn package() {
    for root in ["apps/comic", "content/comic"] {
      let tempdir = tempdir();

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

  #[test]
  fn output_in_root_error() {
    assert_matches!(
      Package {
        root: "foo".into(),
        output: "foo/bar".into(),
      }
      .run()
      .unwrap_err(),
      Error::OutputInRoot {
        output,
        root,
        ..
      }
      if output == "foo/bar" && root == "foo",
    );
  }

  #[test]
  fn output_is_dir_error() {
    let tempdir = tempdir();

    let output_dir = tempdir.path_utf8().join("foo");

    fs::create_dir(&output_dir).unwrap();

    assert_matches!(
      Package {
        root: "foo".into(),
        output: output_dir.clone(),
      }
      .run()
      .unwrap_err(),
      Error::OutputIsDir {
        output,
        ..
      }
      if output == output_dir,
    );
  }

  #[test]
  fn metadata_missing_error() {
    let tempdir = tempdir();

    let root_dir = tempdir.path_utf8().join("root");
    let output = tempdir.path_utf8().join("output.package");

    fs::create_dir(&root_dir).unwrap();

    assert_matches!(
      Package {
        root: root_dir.clone(),
        output,
      }
      .run()
      .unwrap_err(),
      Error::MetadataMissing {
        root,
        ..
      }
      if root == root_dir,
    );
  }

  #[test]
  fn app_requires_index_html() {
    let tempdir = tempdir();

    let root_dir = tempdir.path_utf8().join("root");
    let output = tempdir.path_utf8().join("output.package");

    fs::create_dir(&root_dir).unwrap();

    fs::write(root_dir.join("metadata.yaml"), "type: app\nhandles: comic").unwrap();

    assert_matches!(
      Package {
        root: root_dir.clone(),
        output,
      }
      .run()
      .unwrap_err(),
      Error::Index {
        root,
        ..
      }
      if root == root_dir,
    );
  }

  trait ResultExt<T> {
    fn unwrap_or_display(self) -> T;
  }

  impl<T, E: Display> ResultExt<T> for Result<T, E> {
    fn unwrap_or_display(self) -> T {
      match self {
        Err(err) => {
          panic!("{}", err);
        }
        Ok(ok) => ok,
      }
    }
  }

  #[test]
  fn app_package_includes_all_files() {
    let tempdir = tempdir();

    let root = tempdir.path_utf8().join("root");
    let output = tempdir.path_utf8().join("output.package");

    fs::create_dir(&root).unwrap();

    fs::write(root.join("metadata.yaml"), "type: app\nhandles: comic").unwrap();
    fs::write(root.join("index.html"), "foo").unwrap();
    fs::write(root.join("index.js"), "bar").unwrap();

    Package {
      root: root.clone(),
      output: output.clone(),
    }
    .run()
    .unwrap_or_display();

    let package = super::super::Package::load(&output).unwrap_or_display();

    assert_eq!(package.files.len(), 3);

    let manifest_bytes = {
      let mut buffer = Vec::new();
      ciborium::into_writer(&package.manifest, &mut buffer).unwrap();
      buffer
    };

    let manifest = blake3::hash(&manifest_bytes);

    let Manifest::App { handles, paths } = package.manifest else {
      panic!("unexpected manifest type");
    };

    assert_eq!(handles, Type::Comic);

    let foo = blake3::hash("foo".as_bytes());
    let bar = blake3::hash("bar".as_bytes());

    assert_eq!(paths.len(), 2);
    assert_eq!(paths["index.html"], foo);
    assert_eq!(paths["index.js"], bar);

    assert_eq!(package.files[&foo], "foo".as_bytes());
    assert_eq!(package.files[&bar], "bar".as_bytes());
    assert_eq!(package.files[&manifest], manifest_bytes);
  }

  #[test]
  fn comic_package_includes_all_pages() {
    let tempdir = tempdir();

    let root = tempdir.path_utf8().join("root");
    let output = tempdir.path_utf8().join("output.package");

    fs::create_dir(&root).unwrap();

    fs::write(root.join("metadata.yaml"), "type: comic").unwrap();
    fs::write(root.join("0.jpg"), "foo").unwrap();
    fs::write(root.join("1.jpg"), "bar").unwrap();

    Package {
      root: root.clone(),
      output: output.clone(),
    }
    .run()
    .unwrap_or_display();

    let package = super::super::Package::load(&output).unwrap_or_display();

    assert_eq!(package.files.len(), 3);

    let manifest_bytes = {
      let mut buffer = Vec::new();
      ciborium::into_writer(&package.manifest, &mut buffer).unwrap();
      buffer
    };

    let manifest = blake3::hash(&manifest_bytes);

    let Manifest::Comic { pages } = package.manifest else {
      panic!("unexpected manifest type");
    };

    let foo = blake3::hash("foo".as_bytes());
    let bar = blake3::hash("bar".as_bytes());

    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0], foo);
    assert_eq!(pages[1], bar);

    assert_eq!(package.files[&foo], "foo".as_bytes());
    assert_eq!(package.files[&bar], "bar".as_bytes());
    assert_eq!(package.files[&manifest], manifest_bytes);
  }

  #[test]
  fn directories_are_ignored() {
    let tempdir = tempdir();

    let root_dir = tempdir.path_utf8().join("root");
    let output = tempdir.path_utf8().join("output.package");

    fs::create_dir(&root_dir).unwrap();

    fs::write(root_dir.join("metadata.yaml"), "type: comic").unwrap();
    fs::write(root_dir.join("0.jpg"), "").unwrap();
    fs::create_dir(&root_dir.join("bar")).unwrap();

    Package {
      root: root_dir.clone(),
      output,
    }
    .run()
    .unwrap();
  }

  #[test]
  fn ds_store_files_are_ignored() {
    let tempdir = tempdir();

    let root_dir = tempdir.path_utf8().join("root");
    let output = tempdir.path_utf8().join("output.package");

    fs::create_dir(&root_dir).unwrap();

    fs::write(root_dir.join("metadata.yaml"), "type: comic").unwrap();
    fs::write(root_dir.join("0.jpg"), "").unwrap();
    fs::write(root_dir.join(".DS_Store"), "").unwrap();

    Package {
      root: root_dir.clone(),
      output,
    }
    .run()
    .unwrap();
  }

  #[test]
  fn comic_must_have_pages() {
    let tempdir = tempdir();

    let root_dir = tempdir.path_utf8().join("root");
    let output = tempdir.path_utf8().join("output.package");

    fs::create_dir(&root_dir).unwrap();

    fs::write(root_dir.join("metadata.yaml"), "type: comic").unwrap();

    assert_matches!(
      Package {
        root: root_dir.clone(),
        output,
      }
      .run()
      .unwrap_err(),
      Error::NoPages {
        root,
        ..
      }
      if root == root_dir,
    );
  }
}
