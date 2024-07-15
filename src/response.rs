use super::*;

#[derive(Deserialize, Serialize)]
pub(crate) struct Search(pub(crate) BTreeSet<Hash>);

#[derive(Deserialize, Serialize)]
pub(crate) struct Ping;

#[derive(Deserialize, Serialize)]
pub(crate) struct Get(pub(crate) Option<Vec<u8>>);
