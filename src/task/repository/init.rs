use crate::repository::Repository;
use anyhow::Result;
use napi::bindgen_prelude::*;
use std::path::Path;

pub struct InitRepository {
  path: String,
  bare: bool,
}

impl InitRepository {
  pub fn new<P: AsRef<Path>>(path: P, bare: bool) -> Self {
    Self {
      path: path.as_ref().to_string_lossy().to_string(),
      bare,
    }
  }
}

#[napi]
impl Task for InitRepository {
  type Output = git2::Repository;
  type JsValue = Repository;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    Ok(init(&self.path, self.bare)?)
  }

  fn resolve(&mut self, _: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(output.into())
  }
}

pub(super) fn init<P: AsRef<Path>>(path: P, bare: bool) -> Result<git2::Repository> {
  if bare {
    git2::Repository::init_bare(path)
  } else {
    git2::Repository::init(path)
  }
  .map_err(anyhow::Error::from)
}

#[cfg(test)]
mod tests {
  use crate::task::repository::init::init;
  use tempfile::TempDir;

  #[test]
  fn smoke_init() {
    let td = TempDir::new().unwrap();
    let path = td.path();

    let repo = init(path.to_string_lossy().to_string(), false).unwrap();
    assert!(!repo.is_bare());
  }

  #[test]
  fn smoke_init_bare() {
    let td = TempDir::new().unwrap();
    let path = td.path();

    let repo = init(path.to_string_lossy().to_string(), true).unwrap();
    assert!(repo.is_bare());
    assert!(repo.namespace().is_none());
  }
}
