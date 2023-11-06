#[napi]
#[derive(Copy, Clone)]
pub struct Oid(pub(crate) git2::Oid);

impl From<git2::Oid> for Oid {
  fn from(value: git2::Oid) -> Self {
    Oid(value)
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
