use napi::bindgen_prelude::ClassInstance;
use std::cmp::Ordering;

#[napi]
#[derive(Copy, Clone)]
pub struct Oid(pub(crate) git2::Oid);

impl From<git2::Oid> for Oid {
  fn from(value: git2::Oid) -> Self {
    Oid(value)
  }
}

#[napi]
impl Oid {
  #[allow(clippy::inherent_to_string)]
  #[allow(clippy::wrong_self_convention)]
  #[napi]
  pub fn to_string(&self) -> String {
    self.0.to_string()
  }

  #[napi]
  pub fn cmp(&self, other: ClassInstance<Oid>) -> i32 {
    let ord = self.0.cmp(&other.0);
    match ord {
      Ordering::Less => -1,
      Ordering::Equal => 0,
      Ordering::Greater => 1,
    }
  }
}

#[napi]
pub struct Object {
  inner: git2::Object<'static>,
}

impl From<git2::Object<'_>> for Object {
  fn from(value: git2::Object<'_>) -> Self {
    Self::new(value)
  }
}

impl Object {
  pub(crate) fn new(object: git2::Object<'_>) -> Self {
    Self {
      inner: unsafe { std::mem::transmute(object) },
    }
  }

  pub(crate) fn inner(&self) -> &git2::Object {
    &self.inner
  }
}

#[napi]
impl Object {
  #[allow(clippy::inherent_to_string)]
  #[napi]
  pub fn to_string(&self) -> String {
    self.inner.id().to_string()
  }
}
