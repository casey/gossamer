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
      let captures = dbg!(RE.captures(dbg!(path)))?;

      let app = library.package(captures[1].parse().unwrap())?;
      let content = library.package(captures[2].parse().unwrap())?;

      Some((app, content))
    }

    if let Some((app, content)) = packages(&self.0, path) {
      match app.manifest {
        Manifest::App { target, .. } => {
          let content = content.manifest.ty();

          let matched = match target {
            Target::Library => false,
            Target::Comic => content == Type::Comic,
          };

          if !matched {
            return Err((StatusCode::BAD_REQUEST, format!("content package of type `{content}` cannot be opened by app with target `{target}`")).into_response());
          }
        }
        _ => {
          return Err(
            (
              StatusCode::BAD_REQUEST,
              format!("app package is of type `{}`, not `app`", app.manifest.ty()),
            )
              .into_response(),
          )
        }
      };
    }

    Ok(())
  }
}
