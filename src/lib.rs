mod credentials;
mod reference;
mod repository;
#[cfg(test)]
mod test;

use napi::bindgen_prelude::*;

#[macro_use]
extern crate napi_derive;

#[napi]
#[derive(Clone)]
pub struct Oid(git2::Oid);

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
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}
