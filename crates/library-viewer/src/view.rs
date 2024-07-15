use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum View {
  #[default]
  Default,
  NodeRequest,
  NodeResponse {
    node: api::Node,
  },
  Package {
    handler: Hash,
    content: Hash,
  },
  SearchRequest {
    id: Id,
  },
  SearchResponse {
    id: Id,
    packages: Option<BTreeMap<Hash, Manifest>>,
  },
}
