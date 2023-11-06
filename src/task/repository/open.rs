use crate::repository::Repository;
use anyhow::Result;
use napi::bindgen_prelude::*;
use std::path::Path;

pub struct OpenRepository {
  path: String,
}

impl OpenRepository {
  pub fn new<P: AsRef<Path>>(path: P) -> Self {
    Self {
      path: path.as_ref().to_string_lossy().to_string(),
    }
  }
}

#[napi]
impl Task for OpenRepository {
  type Output = git2::Repository;
  type JsValue = Repository;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    Ok(open(&self.path)?)
  }

  fn resolve(&mut self, _: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(output.into())
  }
}

fn open<P: AsRef<Path>>(path: P) -> Result<git2::Repository> {
  let is_bare = !matches!(path.as_ref().join(".git").try_exists(), Ok(true));
  let repo = if is_bare {
    git2::Repository::open_bare(path)
  } else {
    git2::Repository::open(path)
  };

  repo.map_err(anyhow::Error::from)
}
