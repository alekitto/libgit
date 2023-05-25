use std::ops::Deref;
use napi::bindgen_prelude::SharedReference;
use crate::repository::Repository;

#[napi]
#[derive(Clone)]
pub struct Oid(pub(crate) git2::Oid);

#[napi]
pub struct Object {
    inner: SharedReference<Repository, git2::Object<'static>>,
}

impl Object {
    pub(crate) fn inner(&self) -> &git2::Object {
        self.inner.deref()
    }
}

impl From<SharedReference<Repository, git2::Object<'static>>> for Object {
    fn from(value: SharedReference<Repository, git2::Object<'static>>) -> Self {
        Self {
            inner: value,
        }
    }
}
