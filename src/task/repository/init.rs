use crate::repository::{InitOptions, Repository};
use anyhow::Result;
use git2::RepositoryInitOptions;
use napi::bindgen_prelude::*;
use std::path::Path;

#[derive(Default)]
pub struct InitRepository {
  path: String,
  bare: bool,
  initial_head: Option<String>,
}

impl InitRepository {
  pub fn new<P: AsRef<Path>>(path: P, options: InitOptions) -> Self {
    Self {
      path: path.as_ref().to_string_lossy().to_string(),
      bare: options.bare.unwrap_or(false),
      initial_head: options.initial_head,
    }
  }
}

#[napi]
impl Task for InitRepository {
  type Output = git2::Repository;
  type JsValue = Repository;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    Ok(init(self)?)
  }

  fn resolve(&mut self, _: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(output.into())
  }
}

pub(super) fn init(task: &InitRepository) -> Result<git2::Repository> {
  let bare = task.bare;
  let path: &Path = task.path.as_ref();

  let mut opts = RepositoryInitOptions::new();
  opts.bare(bare);
  if let Some(h) = task.initial_head.as_deref() {
    opts.initial_head(h);
  }

  git2::Repository::init_opts(path, &opts).map_err(anyhow::Error::from)
}

#[cfg(test)]
mod tests {
  use crate::task::repository::init::init;
  use crate::task::InitRepository;
  use tempfile::TempDir;

  #[test]
  fn smoke_init() {
    let td = TempDir::new().unwrap();
    let path = td.path();

    let repo = init(&InitRepository {
      path: path.to_string_lossy().to_string(),
      bare: false,
      ..Default::default()
    })
    .unwrap();
    assert!(!repo.is_bare());
  }

  #[test]
  fn smoke_init_bare() {
    let td = TempDir::new().unwrap();
    let path = td.path();

    let repo = init(&InitRepository {
      path: path.to_string_lossy().to_string(),
      bare: true,
      ..Default::default()
    })
    .unwrap();
    assert!(repo.is_bare());
    assert!(repo.namespace().is_none());
  }
}
