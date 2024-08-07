use super::*;

pub struct Api {
  base: Url,
}

impl Default for Api {
  fn default() -> Self {
    let location = web_sys::window().unwrap().location();
    let mut base = Url::parse(&location.href().unwrap()).unwrap();
    base.set_fragment(None);
    base.set_query(None);
    Self { base }
  }
}

impl Api {
  pub async fn manifest(&self) -> Result<Manifest, Error> {
    self.get("api/manifest").await
  }

  pub async fn packages(&self) -> Result<BTreeMap<Hash, Manifest>, Error> {
    self.get("api/packages").await
  }

  pub async fn handlers(&self) -> Result<BTreeMap<Target, Hash>, Error> {
    self.get("api/handlers").await
  }

  async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, Error> {
    let url = self.base.join(path).unwrap();

    let response = reqwest::Client::new()
      .get(url.clone())
      .send()
      .await
      .with_context(|_| error::Request { url: url.clone() })?;

    let status = response.status();

    ensure!(
      status.is_success(),
      error::Status {
        status,
        url: url.clone()
      }
    );

    let body = response
      .bytes()
      .await
      .with_context(|_| error::Request { url: url.clone() })?;

    ciborium::from_reader(Cursor::new(body))
      .with_context(|_| error::Deserialize { url: url.clone() })
  }
}
