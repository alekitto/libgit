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
  git2::Repository::open(path).map_err(anyhow::Error::from)
}

#[cfg(test)]
mod tests {
  use crate::repository::Repository;
  use crate::task::repository::init::init;
  use crate::task::repository::open::open;
  use std::path::Path;
  use tempfile::TempDir;

  #[test]
  fn smoke_open() {
    let td = TempDir::new().unwrap();
    let path = td.path();

    init(td.path(), false).unwrap();
    let repo = Repository::from(open(path).unwrap());
    assert!(!repo.is_bare());
    assert!(repo.is_empty().unwrap());
    assert_eq!(
      crate::test::realpath(Path::new(&repo.path())).unwrap(),
      crate::test::realpath(&td.path().join(".git/")).unwrap()
    );
    assert_eq!(repo.state(), crate::RepositoryState::Clean);
  }

  #[test]
  fn smoke_open_bare() {
    let td = TempDir::new().unwrap();
    let path = td.path();
    init(td.path(), true).unwrap();

    let repo = Repository::from(open(path).unwrap());
    assert!(repo.is_bare());
    assert_eq!(
      crate::test::realpath(Path::new(&repo.path())).unwrap(),
      crate::test::realpath(&td.path().join("")).unwrap()
    );
  }
}
