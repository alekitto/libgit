mod commit;
mod credentials;
mod fetch_options;
mod object;
mod reference;
mod repository;
mod task;

use napi::bindgen_prelude::*;

#[cfg(test)]
mod test;

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

#[napi]
pub enum ResetType {
  Soft,
  Hard,
  Mixed,
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
