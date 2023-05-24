use crate::repository::Repository;
use crate::Oid;
use anyhow::Result;
use napi::bindgen_prelude::*;

#[napi]
pub struct Commit {
  repository: Reference<Repository>,
  pub(crate) inner: SharedReference<Repository, git2::Commit<'static>>,
}

#[napi]
impl Commit {
  pub fn find(repository: Reference<Repository>, target: Oid, env: Env) -> Result<Self> {
    let inner = repository.clone(env)?.share_with(env, |repository| {
      let commit = repository
        .repository
        .find_commit(target.0)
        .map_err(anyhow::Error::from)?;

      Ok(commit)
    })?;

    Ok(Self { repository, inner })
  }

  pub fn oid(&self) -> Oid {
    Oid(self.inner.id())
  }
}
