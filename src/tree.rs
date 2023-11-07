use crate::object::Object;
use crate::repository::Repository;
use napi::bindgen_prelude::*;
use std::path::Path;

#[napi]
#[derive(Clone)]
pub struct Tree {
  inner: git2::Tree<'static>,
}

unsafe impl Send for Tree {}

impl From<git2::Tree<'_>> for Tree {
  fn from(value: git2::Tree) -> Self {
    Self {
      inner: unsafe { std::mem::transmute(value) },
    }
  }
}

impl From<Tree> for git2::Tree<'_> {
  fn from(value: Tree) -> Self {
    value.inner
  }
}

#[napi]
impl Tree {
  pub fn entry_by_path(&self, path: String) -> napi::Result<TreeEntry> {
    Ok(
      self
        .inner
        .get_path(Path::new(&path))
        .map(TreeEntry::from)
        .map_err(anyhow::Error::from)?,
    )
  }
}

#[napi]
pub struct TreeEntry {
  inner: git2::TreeEntry<'static>,
}

#[napi]
impl TreeEntry {
  #[napi]
  pub fn is_tree(&self) -> bool {
    matches!(self.inner.kind(), Some(git2::ObjectType::Tree))
  }

  #[napi]
  pub fn to_object(&self, repository: ClassInstance<Repository>) -> Result<Object> {
    let repository = futures::executor::block_on(repository.repository.lock());
    Ok(
      self
        .inner
        .to_object(&repository)
        .map(Object::from)
        .map_err(anyhow::Error::from)?,
    )
  }
}

impl From<git2::TreeEntry<'_>> for TreeEntry {
  fn from(value: git2::TreeEntry) -> Self {
    Self {
      inner: unsafe { std::mem::transmute(value) },
    }
  }
}
