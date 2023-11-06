mod commit;
mod credentials;
mod fetch_options;
mod object;
mod reference;
mod remote;
mod repository;
mod task;

use napi::bindgen_prelude::*;

#[macro_use]
extern crate napi_derive;

#[napi]
#[derive(Debug, Eq, PartialEq)]
pub enum RepositoryState {
  Clean,
  Merge,
  Revert,
  RevertSequence,
  CherryPick,
  CherryPickSequence,
  Bisect,
  Rebase,
  RebaseInteractive,
  RebaseMerge,
  ApplyMailbox,
  ApplyMailboxOrRebase,
}

impl From<git2::RepositoryState> for RepositoryState {
  fn from(value: git2::RepositoryState) -> Self {
    match value {
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
    }
  }
}

#[napi]
pub enum ResetType {
  Soft,
  Hard,
  Mixed,
}

#[napi]
pub enum Direction {
  Fetch,
  Push,
}

impl From<ResetType> for git2::ResetType {
  fn from(value: ResetType) -> Self {
    match value {
      ResetType::Soft => git2::ResetType::Soft,
      ResetType::Hard => git2::ResetType::Hard,
      ResetType::Mixed => git2::ResetType::Mixed,
    }
  }
}
