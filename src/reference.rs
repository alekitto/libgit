use crate::Oid;
use napi::bindgen_prelude::*;

#[napi]
#[derive(PartialEq)]
pub enum ReferenceType {
  Direct = 1,
  Symbolic = 2,
}

#[napi]
pub struct Reference {
  pub(crate) kind: ReferenceType,
  pub(crate) target: Option<Oid>,
  pub(crate) name: Option<String>,
}

#[napi]
impl Reference {
  pub fn kind(&self) -> ReferenceType {
    self.kind
  }

  pub fn target(&self) -> Option<Oid> {
    self.target.clone()
  }

  pub fn name(&self) -> Option<String> {
    self.name.clone()
  }
}
