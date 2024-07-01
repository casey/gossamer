use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Manifest {
  App { paths: BTreeMap<String, Hash> },
  Comic { pages: Vec<Hash> },
}
