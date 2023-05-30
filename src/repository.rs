use crate::commit::Commit;
use crate::fetch_options::FetchOptions;
use crate::object::Oid;
use crate::reference::ReferenceType;
use crate::task::{
  CloneRepository, FetchRepository, InitRepository, OpenRepository, RepositoryStatus,
};
use crate::ResetType;
use anyhow::Result;
use napi::bindgen_prelude::*;
use napi::Env;

#[napi]
pub struct Repository {
  pub(crate) repository: git2::Repository,
}

impl From<git2::Repository> for Repository {
  fn from(value: git2::Repository) -> Self {
    Self { repository: value }
  }
}

#[napi]
impl Repository {
  #[napi(ts_return_type = "Promise<Repository>")]
  pub fn init(path: String, bare: Option<bool>) -> AsyncTask<InitRepository> {
    AsyncTask::new(InitRepository::new(path, bare.unwrap_or(false)))
  }

  #[napi(ts_return_type = "Promise<Repository>")]
  pub fn open(path: String) -> AsyncTask<OpenRepository> {
    AsyncTask::new(OpenRepository::new(path))
  }

  #[napi(ts_return_type = "Promise<Repository>")]
  pub fn clone(
    url: String,
    path: String,
    recursive: Option<bool>,
    fetch_options: Option<FetchOptions>,
    env: Env,
  ) -> Result<AsyncTask<CloneRepository>> {
    Ok(AsyncTask::new(CloneRepository::new(
      &url,
      path,
      recursive.unwrap_or(false),
      fetch_options.unwrap_or_default().into_fetch_opts(&env)?,
    )))
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

  #[napi(ts_return_type = "Promise<RepositoryState>")]
  pub fn state(&self, this: Reference<Repository>) -> AsyncTask<RepositoryStatus> {
    AsyncTask::new(RepositoryStatus::new(this))
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

  #[napi(ts_return_type = "Promise<void>")]
  pub fn fetch(
    &self,
    options: Option<FetchOptions>,
    env: Env,
    repo: Reference<Repository>,
  ) -> Result<AsyncTask<FetchRepository>> {
    let opts = options.unwrap_or_default();
    Ok(AsyncTask::new(FetchRepository::new(
      repo,
      opts.into_fetch_opts(&env)?,
    )))
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
  pub fn reset(
    &self,
    target: Either3<
      ClassInstance<Commit>,
      ClassInstance<crate::reference::Reference>,
      ClassInstance<Oid>,
    >,
    reset_type: Option<ResetType>,
    env: Env,
    this_ref: Reference<Repository>,
  ) -> Result<()> {
    let object = match target {
      Either3::A(commit) => (*commit).as_object(env),
      Either3::B(reference) => {
        let oid = reference
          .target()
          .ok_or_else(|| anyhow::Error::msg("Cannot find reference target"))?;
        Ok(crate::object::Object::from(Self::shared_object_from_oid(
          oid.0, env, this_ref,
        )?))
      }
      Either3::C(oid) => Ok(crate::object::Object::from(Self::shared_object_from_oid(
        oid.0, env, this_ref,
      )?)),
    }?;

    self.repository.reset(
      object.inner(),
      reset_type.unwrap_or(ResetType::Mixed).into(),
      None,
    )?;

    Ok(())
  }

  fn shared_object_from_oid(
    oid: git2::Oid,
    env: Env,
    this_ref: Reference<Repository>,
  ) -> Result<SharedReference<Repository, git2::Object<'static>>> {
    Ok(this_ref.share_with(env, |repo| {
      let object = repo
        .repository
        .find_object(oid, None)
        .map_err(anyhow::Error::from)?;
      match object.kind() {
        Some(git2::ObjectType::Commit) | Some(git2::ObjectType::Tag) => Ok(object),
        _ => Err(anyhow::Error::msg("Invalid object type").into()),
      }
    })?)
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
