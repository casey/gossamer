use super::*;

#[derive(Parser)]
pub(crate) struct Package {
  #[arg(long, help = "Package contents of directory <ROOT>.")]
  pub(crate) root: Utf8PathBuf,
  #[arg(long, help = "Save package to <OUTPUT>.")]
  pub(crate) output: Utf8PathBuf,
}

impl Package {
  pub(crate) fn run(self) -> Result {
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

    super::Package::save(hashes, &manifest, &self.output, &self.root)
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

      hashes.insert(
        relative.clone(),
        (Hash::reader(file).context(context)?, len),
      );
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

  #[test]
  fn package() {
    for root in ["tests/packages/app-comic", "tests/packages/comic"] {
      let tempdir = tempdir();

      let result = Package {
        root: root.into(),
        output: tempdir.join("output.package"),
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

    let output_dir = tempdir.join("foo");

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

    let root_dir = tempdir.join("root");
    let output = tempdir.join("output.package");

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
  fn comic_package_includes_all_pages() {
    let tempdir = tempdir();

    let root = tempdir.join("root");
    let output = tempdir.join("output.package");

    tempdir.write_yaml(
      "root/metadata.yaml",
      Metadata {
        name: "comic".into(),
        media: metadata::Media::Comic,
      },
    );

    tempdir.write("root/0.jpg", "foo");
    tempdir.write("root/1.jpg", "bar");

    Package {
      root,
      output: output.clone(),
    }
    .run()
    .unwrap_or_display();

    let package = super::super::Package::load(&output).unwrap_or_display();

    assert_eq!(package.files.len(), 3);

    let manifest_bytes = package.manifest.to_cbor();

    let manifest = Hash::bytes(&manifest_bytes);

    let Media::Comic { pages } = package.manifest.media else {
      panic!("unexpected manifest type");
    };

    let foo = Hash::bytes("foo".as_bytes());
    let bar = Hash::bytes("bar".as_bytes());

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

    let root = tempdir.join("root");
    let output = tempdir.join("output.package");

    tempdir.write_yaml(
      "root/metadata.yaml",
      Metadata {
        name: "comic".into(),
        media: metadata::Media::Comic,
      },
    );

    tempdir.touch("root/0.jpg");

    fs::create_dir(root.join("bar")).unwrap();

    Package { root, output }.run().unwrap();
  }

  #[test]
  fn ds_store_files_are_ignored() {
    let tempdir = tempdir();

    let root = tempdir.join("root");
    let output = tempdir.join("output.package");

    tempdir.write_yaml(
      "root/metadata.yaml",
      Metadata {
        name: "comic".into(),
        media: metadata::Media::Comic,
      },
    );

    tempdir.touch("root/0.jpg");
    tempdir.touch("root/.DS_Store");

    Package { root, output }.run().unwrap();
  }

  #[test]
  fn comic_must_have_pages() {
    let tempdir = tempdir();

    let root_dir = tempdir.join("root");
    let output = tempdir.join("output.package");

    tempdir.write_yaml(
      "root/metadata.yaml",
      Metadata {
        name: "comic".into(),
        media: metadata::Media::Comic,
      },
    );

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

  #[test]
  fn comic_page_missing_error() {
    let tempdir = tempdir();

    let root = tempdir.join("root");
    let output = tempdir.join("output.package");

    tempdir.write_yaml(
      "root/metadata.yaml",
      Metadata {
        name: "comic".into(),
        media: metadata::Media::Comic,
      },
    );

    tempdir.touch("root/1.jpg");

    assert_matches!(
      Package {
        root,
        output,
      }
      .run()
      .unwrap_err(),
      Error::PageMissing {
        page,
        ..
      }
      if page == 0,
    );
  }

  #[test]
  fn comic_page_duplicated_error() {
    let tempdir = tempdir();

    let root = tempdir.path_utf8().join("root");
    let output = tempdir.path_utf8().join("output.package");

    tempdir.write_yaml(
      "root/metadata.yaml",
      Metadata {
        name: "comic".into(),
        media: metadata::Media::Comic,
      },
    );

    tempdir.touch("root/0.jpg");
    tempdir.touch("root/00.jpg");

    assert_matches!(
      Package {
        root,
        output,
      }
      .run()
      .unwrap_err(),
      Error::PageDuplicated {
        page,
        ..
      }
      if page == 0,
    );
  }

  #[test]
  fn comic_unexpected_file() {
    let tempdir = tempdir();

    let root = tempdir.join("root");
    let output = tempdir.join("output.package");

    tempdir.write_yaml(
      "root/metadata.yaml",
      Metadata {
        name: "comic".into(),
        media: metadata::Media::Comic,
      },
    );

    tempdir.touch("root/0.jpg");
    tempdir.touch("root/foo.jpg");

    assert_matches!(
      Package {
        root,
        output,
      }
      .run()
      .unwrap_err(),
      Error::UnexpectedFile {
        file,
        ty,
        ..
      }
      if file == "foo.jpg" && ty == Type::Comic,
    );
  }

  #[test]
  fn comic_invalid_page() {
    let tempdir = tempdir();

    let root = tempdir.join("root");
    let output = tempdir.join("output.package");

    tempdir.write_yaml(
      "root/metadata.yaml",
      Metadata {
        name: "comic".into(),
        media: metadata::Media::Comic,
      },
    );

    tempdir.touch(format!("root/{}.jpg", u128::from(u64::MAX) + 1));

    assert_matches!(
      Package {
        root,
        output,
      }
      .run()
      .unwrap_err(),
      Error::InvalidPage {
        path,
        ..
      }
      if path == "18446744073709551616.jpg",
    );
  }
}
