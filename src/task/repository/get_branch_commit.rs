use crate::commit::Commit;
use crate::object::Oid;
use crate::repository::Repository;
use anyhow::anyhow;
use napi::bindgen_prelude::Reference;
use napi::{Env, Task};

pub enum BranchNameRef {
  Name(String),
  Reference(Oid),
}

pub struct GetBranchCommit {
  repository: Reference<Repository>,
  name: BranchNameRef,
}

impl GetBranchCommit {
  pub fn new(repository: Reference<Repository>, name: BranchNameRef) -> Self {
    Self { repository, name }
  }
}

#[napi]
impl Task for GetBranchCommit {
  type Output = Oid;
  type JsValue = Commit;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    let oid = match &self.name {
      BranchNameRef::Name(name) => {
        futures::executor::block_on(self.repository.get_reference(name.clone()))?.target()
      }
      BranchNameRef::Reference(oid) => Some(*oid),
    };

    oid.ok_or_else(|| anyhow!("not a commit reference").into())
  }

  fn resolve(&mut self, _: Env, value: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(futures::executor::block_on(
      self.repository.internal_find_commit(value),
    )?)
  }
}
