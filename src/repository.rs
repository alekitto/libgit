use crate::commit::Commit;
use crate::credentials::Credentials;
use crate::reference::ReferenceType;
use crate::{Oid, RepositoryState};
use anyhow::Result;
use napi::bindgen_prelude::*;
use napi::{Env, JsFunction, JsUnknown};
use std::path::Path;

#[napi(object)]
#[derive(Default)]
pub struct FetchOptions {
  pub remote: Option<String>,
  pub prune: Option<bool>,
  #[napi(ts_type = "(url: string, username?: string) => Credentials")]
  pub credentials_callback: Option<JsFunction>,
  pub skip_certificate_check: Option<bool>,
}

#[napi]
pub struct Repository {
  pub(crate) repository: git2::Repository,
}

#[napi]
impl Repository {
  #[napi(factory, js_name = "init")]
  pub fn js_init(path: String, bare: Option<bool>) -> Result<Self> {
    Self::init(path, bare.unwrap_or(false))
  }

  #[napi(factory, js_name = "open")]
  pub fn js_open(path: String) -> Result<Self> {
    Self::open(path)
  }

  #[napi(factory, js_name = "clone")]
  pub fn js_clone(url: String, path: String, recursive: Option<bool>) -> Result<Self> {
    Self::clone(&url, path, recursive.unwrap_or(false))
  }

  pub fn init<P: AsRef<Path>>(path: P, bare: bool) -> Result<Self> {
    let repository = if bare {
      git2::Repository::init_bare(path)?
    } else {
      git2::Repository::init(path)?
    };

    Ok(Self { repository })
  }

  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
    let repository = git2::Repository::open(path)?;

    Ok(Self { repository })
  }

  pub fn clone<P: AsRef<Path>>(url: &str, directory: P, recursive: bool) -> Result<Self> {
    let repository = if recursive {
      git2::Repository::clone_recurse(url, directory)
    } else {
      git2::Repository::clone(url, directory)
    }?;

    Ok(Self { repository })
  }

  #[napi]
  pub fn namespace(&self) -> Option<String> {
    self.repository.namespace().map(ToString::to_string)
  }

  #[napi]
  pub fn is_bare(&self) -> bool {
    self.repository.is_bare()
  }

  #[napi]
  pub fn is_empty(&self) -> Result<bool> {
    Ok(self.repository.is_empty()?)
  }

  #[napi]
  pub fn path(&self) -> String {
    self.repository.path().to_string_lossy().to_string()
  }

  #[napi]
  pub fn state(&self) -> RepositoryState {
    match self.repository.state() {
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

  #[napi]
  pub fn find_commit(
    &self,
    target: ClassInstance<Oid>,
    reference: Reference<Repository>,
    env: Env,
  ) -> Result<Commit> {
    Commit::find(reference, target.clone(), env)
  }

  #[napi]
  pub fn create_branch(
    &self,
    name: String,
    commit: Either3<ClassInstance<Commit>, ClassInstance<Oid>, String>,
    force: bool,
    this_ref: Reference<Repository>,
    env: Env,
  ) -> Result<crate::reference::Reference> {
    let commit = match commit {
      Either3::A(commit) => Commit::find(this_ref.clone(env)?, commit.oid(), env)?,
      Either3::B(oid) => Commit::find(this_ref.clone(env)?, oid.as_ref().clone(), env)?,
      Either3::C(commit_sha) => {
        let oid = git2::Oid::from_str(&commit_sha)?;
        Commit::find(this_ref.clone(env)?, Oid(oid), env)?
      }
    };

    let inner = this_ref.share_with(env, |repository| {
      Ok(
        repository
          .repository
          .branch(&name, &commit.inner, force)
          .map_err(anyhow::Error::from)?
          .into_reference(),
      )
    })?;

    Ok(crate::reference::Reference { inner })
  }

  #[napi]
  pub fn fetch(&self, options: Option<FetchOptions>, env: Env) -> Result<()> {
    let options = options.unwrap_or_default();
    let remote_name = options.remote.unwrap_or_else(|| "origin".to_string());

    let mut cb = git2::RemoteCallbacks::default();
    if let Some(cred_cb) = &options.credentials_callback {
      cb.credentials(move |url, username, _| {
        (|| -> Result<ClassInstance<Credentials>, anyhow::Error> {
          let credentials = cred_cb.call::<JsUnknown>(
            None,
            &[
              env.create_string(url)?.into_unknown(),
              if let Some(username) = username {
                env.create_string(username)?.into_unknown()
              } else {
                env.get_undefined()?.into_unknown()
              },
            ],
          )?;

          Ok(ClassInstance::from_unknown(credentials)?)
        })()
        .and_then(|c| c.to_cred())
        .map_err(|err| git2::Error::from_str(&err.to_string()))
      });
    }

    if options.skip_certificate_check.unwrap_or(false) {
      cb.certificate_check(|_cert, _domain| Ok(git2::CertificateCheckStatus::CertificateOk));
    }

    let mut remote = self
      .repository
      .find_remote(&remote_name)
      .or_else(|_| self.repository.remote_anonymous(&remote_name))?;

    let mut fo = git2::FetchOptions::default();
    fo.remote_callbacks(cb);
    remote.download(&[] as &[&str], Some(&mut fo))?;
    remote.disconnect()?;

    remote.update_tips(None, true, git2::AutotagOption::Unspecified, None)?;
    if options.prune.unwrap_or(false) {
      remote.prune(None)?;
    }

    Ok(())
  }

  #[napi]
  pub fn get_current_branch(
    &self,
    this_ref: Reference<Repository>,
    env: Env,
  ) -> Result<crate::reference::Reference> {
    self.head(this_ref, env)
  }

  #[napi]
  pub fn head(
    &self,
    this_ref: Reference<Repository>,
    env: Env,
  ) -> Result<crate::reference::Reference> {
    let inner = this_ref.share_with(env, |repository| {
      Ok(repository.repository.head().map_err(anyhow::Error::from)?)
    })?;

    Ok(crate::reference::Reference { inner })
  }

  #[napi]
  pub fn get_reference(
    &self,
    reference: String,
    this_ref: Reference<Repository>,
    env: Env,
  ) -> Result<crate::reference::Reference> {
    let inner = this_ref.share_with(env, |repository| {
      Ok(
        repository
          .repository
          .find_reference(&reference)
          .map_err(anyhow::Error::from)?,
      )
    })?;

    Ok(crate::reference::Reference { inner })
  }

  #[napi]
  pub fn get_reference_names(&self, reference_type: Option<ReferenceType>) -> Result<Vec<String>> {
    Ok(
      self
        .repository
        .references()?
        .filter_map(|r| {
          if let Ok(r) = r {
            r.name().map(|n| (r.kind(), n.to_string()))
          } else {
            None
          }
        })
        .filter_map(|(kind, name)| {
          if let Some(rt) = reference_type {
            if (rt == ReferenceType::Direct && kind == Some(git2::ReferenceType::Direct))
              || (rt == ReferenceType::Symbolic && kind == Some(git2::ReferenceType::Symbolic))
            {
              Some(name)
            } else {
              None
            }
          } else {
            None
          }
        })
        .collect(),
    )
  }
}

#[cfg(test)]
mod tests {
  use crate::repository::Repository;
  use std::path::Path;
  use tempfile::TempDir;

  #[test]
  fn smoke_init() {
    let td = TempDir::new().unwrap();
    let path = td.path();

    let repo = Repository::init(path, false).unwrap();
    assert!(!repo.is_bare());
  }

  #[test]
  fn smoke_init_bare() {
    let td = TempDir::new().unwrap();
    let path = td.path();

    let repo = Repository::init(path, true).unwrap();
    assert!(repo.is_bare());
    assert!(repo.namespace().is_none());
  }

  #[test]
  fn smoke_open() {
    let td = TempDir::new().unwrap();
    let path = td.path();

    Repository::init(td.path(), false).unwrap();
    let repo = Repository::open(path).unwrap();
    assert!(!repo.is_bare());
    assert!(repo.is_empty().unwrap());
    assert_eq!(
      crate::test::realpath(Path::new(&repo.path())).unwrap(),
      crate::test::realpath(&td.path().join(".git/")).unwrap()
    );
    assert_eq!(repo.state(), crate::RepositoryState::Clean);
  }

  #[test]
  fn smoke_open_bare() {
    let td = TempDir::new().unwrap();
    let path = td.path();
    Repository::init(td.path(), true).unwrap();

    let repo = Repository::open(path).unwrap();
    assert!(repo.is_bare());
    assert_eq!(
      crate::test::realpath(Path::new(&repo.path())).unwrap(),
      crate::test::realpath(&td.path().join("")).unwrap()
    );
  }
}
