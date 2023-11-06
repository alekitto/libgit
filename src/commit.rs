use crate::object::{Object, Oid};
use crate::repository::Repository;
use napi::bindgen_prelude::*;

#[napi]
#[derive(Clone)]
pub struct Commit {
  id: git2::Oid,
  tree_id: git2::Oid,
}

impl Commit {
  pub(crate) fn from_raw(raw: git2::Commit) -> Self {
    Self {
      id: raw.id(),
      tree_id: raw.tree_id(),
    }
  }
}

#[napi]
impl Commit {
  #[napi]
  pub async fn as_object(&self, repository: &Repository) -> Result<Object> {
    let lock = repository.repository.lock().await;
    let commit = lock.find_commit(self.id).map_err(anyhow::Error::from)?;

    Ok(Object::new(commit.as_object().clone()))
  }

  #[napi]
  pub fn oid(&self) -> Oid {
    Oid(self.id)
  }
}
