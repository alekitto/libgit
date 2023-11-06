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
  inner: git2::Reference<'static>,
}

impl Reference {
  pub(crate) fn new(object: git2::Reference<'_>) -> Self {
    Self {
      inner: unsafe { std::mem::transmute(object) },
    }
  }
}

#[napi]
impl Reference {
  #[napi]
  pub fn kind(&self) -> ReferenceType {
    if self.inner.kind() == Some(git2::ReferenceType::Direct) {
      ReferenceType::Direct
    } else {
      ReferenceType::Symbolic
    }
  }

  #[napi]
  pub fn target(&self) -> Option<Oid> {
    self.inner.target().map(Oid)
  }

  #[napi]
  pub fn name(&self) -> Option<String> {
    self.inner.name().map(ToString::to_string)
  }
}
