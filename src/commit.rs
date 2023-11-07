use crate::object::{Object, Oid};
use crate::tree::Tree;
use napi::bindgen_prelude::*;

#[napi]
#[derive(Clone)]
pub struct Commit {
  inner: git2::Commit<'static>,
}

impl From<git2::Commit<'_>> for Commit {
  fn from(raw: git2::Commit) -> Self {
    Self {
      inner: unsafe { std::mem::transmute(raw) },
    }
  }
}

#[napi]
impl Commit {
  #[napi]
  pub fn as_object(&self) -> Object {
    Object::new(self.inner.as_object().clone())
  }

  #[napi]
  pub fn oid(&self) -> Oid {
    Oid(self.inner.id())
  }

  #[napi]
  pub fn message_raw(&self) -> Option<String> {
    self.inner.message_raw().map(|s| s.to_string())
  }

  #[napi]
  pub fn author(&self) -> Signature {
    self.inner.author().into()
  }

  #[napi]
  pub fn committer(&self) -> Signature {
    self.inner.committer().into()
  }

  #[napi]
  pub fn get_parents(&self) -> Vec<Commit> {
    self.inner.parents().map(|p| p.into()).collect()
  }

  #[napi]
  pub fn get_tree(&self) -> Result<Tree> {
    Ok(
      self
        .inner
        .tree()
        .map(Tree::from)
        .map_err(anyhow::Error::from)?,
    )
  }
}

impl From<Commit> for git2::Commit<'_> {
  fn from(value: Commit) -> Self {
    value.inner
  }
}

#[napi]
#[derive(Copy, Clone)]
pub struct Time {
  pub time: i64,
  pub offset: i32,
}

#[napi]
#[derive(Clone)]
pub struct Signature {
  name: Option<String>,
  email: Option<String>,
  time: Time,
}

#[napi]
impl Signature {
  #[napi]
  pub fn name(&self) -> Option<String> {
    self.name.clone()
  }

  #[napi]
  pub fn email(&self) -> Option<String> {
    self.email.clone()
  }

  #[napi]
  pub fn time(&self) -> Time {
    self.time
  }
}

impl From<git2::Signature<'_>> for Signature {
  fn from(value: git2::Signature) -> Self {
    Self {
      name: value.name().map(|s| s.to_string()),
      email: value.email().map(|s| s.to_string()),
      time: Time {
        time: value.when().seconds(),
        offset: value.when().offset_minutes(),
      },
    }
  }
}

impl TryFrom<Signature> for git2::Signature<'_> {
  type Error = git2::Error;

  fn try_from(value: Signature) -> std::result::Result<Self, Self::Error> {
    git2::Signature::new(
      value.name.as_deref().unwrap_or_default(),
      value.email.as_deref().unwrap_or_default(),
      &git2::Time::new(value.time.time, value.time.offset),
    )
  }
}
