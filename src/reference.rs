use crate::repository::Repository;
use crate::object::Oid;
use napi::bindgen_prelude::*;

#[napi]
#[derive(PartialEq)]
pub enum ReferenceType {
  Direct = 1,
  Symbolic = 2,
}

#[napi]
pub struct Reference {
  pub(crate) inner: SharedReference<Repository, git2::Reference<'static>>,
}

#[napi]
impl Reference {
  pub fn kind(&self) -> ReferenceType {
    if self.inner.kind() == Some(git2::ReferenceType::Direct) {
      ReferenceType::Direct
    } else {
      ReferenceType::Symbolic
    }
  }

  pub fn target(&self) -> Option<Oid> {
    self.inner.target().map(Oid)
  }

  pub fn name(&self) -> Option<String> {
    self.inner.name().map(ToString::to_string)
  }
}

impl From<SharedReference<Repository, git2::Reference<'static>>> for Reference {
  fn from(value: SharedReference<Repository, git2::Reference<'static>>) -> Self {
    Self { inner: value }
  }
}
