use super::*;

#[derive(Clone)]
pub struct TargetValidator(pub Arc<Library>);

impl ValidateRequest<Body> for TargetValidator {
  type ResponseBody = Body;

  fn validate(
    &mut self,
    request: &mut http::Request<Body>,
  ) -> Result<(), Response<Self::ResponseBody>> {
    let path = request.uri().path();

    static RE: Lazy<Regex> = lazy_regex!("^/([[:xdigit:]]{64})/([[:xdigit:]]{64})/.*$");

    fn packages<'a>(library: &'a Library, path: &str) -> Option<(&'a Package, &'a Package)> {
      let captures = RE.captures(path)?;

      let app = library.package(captures[1].parse().unwrap())?;
      let content = library.package(captures[2].parse().unwrap())?;

      Some((app, content))
    }

    if let Some((app, content)) = packages(&self.0, path) {
      match app.manifest.media {
        Media::App { target, .. } => {
          let content = content.manifest.media.ty();

          let matched = match target {
            Target::App => content == Type::App,
            Target::Comic => content == Type::Comic,
            Target::Root => false,
          };

          if !matched {
            return Err((StatusCode::BAD_REQUEST, format!("content package of type `{content}` cannot be opened by app with target `{target}`")).into_response());
          }
        }
        _ => {
          return Err(
            (
              StatusCode::BAD_REQUEST,
              format!(
                "app package is of type `{}`, not `app`",
                app.manifest.media.ty()
              ),
            )
              .into_response(),
          )
        }
      };
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn validator() {
    let app_package = PACKAGES.app();
    let content_package = PACKAGES.comic();
    let root_package = PACKAGES.root();

    let app = app_package.hash;
    let content = content_package.hash;
    let root = root_package.hash;

    let mut library = Library::default();

    library.add(app_package.clone());
    library.add(content_package.clone());
    library.add(root_package.clone());

    let library = Arc::new(library);

    let mut validator = TargetValidator(library);

    let mut request = http::Request::builder()
      .method("GET")
      .uri("https://example.com/")
      .body(Body::empty())
      .unwrap();

    validator.validate(&mut request).unwrap();

    let mut request = http::Request::builder()
      .method("GET")
      .uri(format!("https://example.com/{app}/{content}/"))
      .body(Body::empty())
      .unwrap();

    validator.validate(&mut request).unwrap();

    let mut request = http::Request::builder()
      .method("GET")
      .uri(format!("https://example.com/{root}/{content}/"))
      .body(Body::empty())
      .unwrap();

    validator.validate(&mut request).unwrap_err();

    let mut request = http::Request::builder()
      .method("GET")
      .uri(format!("https://example.com/{content}/{content}/"))
      .body(Body::empty())
      .unwrap();

    validator.validate(&mut request).unwrap_err();
  }
}
