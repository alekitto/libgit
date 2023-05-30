use crate::repository::Repository;
use crate::RepositoryState;
use napi::bindgen_prelude::*;

pub struct RepositoryStatus {
  repository: Reference<Repository>,
}

impl RepositoryStatus {
  pub fn new(repository: Reference<Repository>) -> Self {
    Self { repository }
  }
}

#[napi]
impl Task for RepositoryStatus {
  type Output = git2::RepositoryState;
  type JsValue = RepositoryState;

  fn compute(&mut self) -> Result<Self::Output> {
    Ok(self.repository.repository.state())
  }

  fn resolve(&mut self, _: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(match output {
      git2::RepositoryState::Clean => RepositoryState::Clean,
      git2::RepositoryState::Merge => RepositoryState::Merge,
      git2::RepositoryState::Revert => RepositoryState::Revert,
      git2::RepositoryState::RevertSequence => RepositoryState::RevertSequence,
      git2::RepositoryState::CherryPick => RepositoryState::CherryPick,
      git2::RepositoryState::CherryPickSequence => RepositoryState::CherryPickSequence,
      git2::RepositoryState::Bisect => RepositoryState::Bisect,
      git2::RepositoryState::Rebase => RepositoryState::Rebase,
      git2::RepositoryState::RebaseInteractive => RepositoryState::RebaseInteractive,
      git2::RepositoryState::RebaseMerge => RepositoryState::RebaseMerge,
      git2::RepositoryState::ApplyMailbox => RepositoryState::ApplyMailbox,
      git2::RepositoryState::ApplyMailboxOrRebase => RepositoryState::ApplyMailboxOrRebase,
    })
  }
}
