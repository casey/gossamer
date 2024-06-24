use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub enum Manifest {
  App { paths: BTreeMap<String, Hash> },
  Comic { pages: Vec<Hash> },
}
