use crate::commit::{Commit, Signature};
use crate::object::Oid;
use crate::repository::Repository;
use crate::tree::Tree;
use napi::bindgen_prelude::{ClassInstance, Reference};
use napi::{Env, Task};

pub struct CreateCommit {
  repository: Reference<Repository>,
  update_ref: Option<String>,
  author: Signature,
  committer: Signature,
  message: String,
  tree: Tree,
  parents: Vec<Commit>,
}

impl CreateCommit {
  pub fn new(
    repository: Reference<Repository>,
    update_ref: Option<String>,
    author: ClassInstance<Signature>,
    committer: ClassInstance<Signature>,
    message: String,
    tree: ClassInstance<Tree>,
    parents: Vec<ClassInstance<Commit>>,
  ) -> Self {
    Self {
      repository,
      update_ref,
      author: author.as_ref().clone(),
      committer: committer.as_ref().clone(),
      message,
      tree: tree.as_ref().clone(),
      parents: parents.iter().map(|c| c.as_ref().clone()).collect(),
    }
  }
}

#[napi]
impl Task for CreateCommit {
  type Output = Oid;
  type JsValue = Oid;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    Ok(futures::executor::block_on(
      self.repository.internal_create_commit(
        self.update_ref.clone(),
        self.author.clone(),
        self.committer.clone(),
        self.message.clone(),
        self.tree.clone(),
        self.parents.clone(),
      ),
    )?)
  }

  fn resolve(&mut self, _: Env, value: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(value)
  }
}
