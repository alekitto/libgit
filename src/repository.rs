use crate::commit::{Commit, Signature};
use crate::config::Config;
use crate::fetch_options::FetchOptions;
use crate::index::Index;
use crate::object::Oid;
use crate::reference::ReferenceType;
use crate::remote::Remote;
use crate::revwalk::Revwalk;
use crate::task::{
  BranchNameRef, CloneRepository, CreateCommit, FetchRepository, GetBranchCommit, InitRepository,
  OpenRepository,
};
use crate::tree::Tree;
use crate::{RepositoryState, ResetType};
use anyhow::anyhow;
use git2::build::CheckoutBuilder;
use napi::bindgen_prelude::*;
use napi::tokio::sync::Mutex;
use napi::{Env, JsObject};

#[napi]
pub struct Repository {
  pub(crate) repository: Mutex<git2::Repository>,
  is_bare: bool,
  path: String,
}

impl From<git2::Repository> for Repository {
  fn from(value: git2::Repository) -> Self {
    Self::new(value)
  }
}

impl Repository {
  pub(crate) fn new(repository: git2::Repository) -> Self {
    let is_bare = repository.is_bare();
    let path = repository.path().to_string_lossy().to_string();

    Self {
      repository: Mutex::new(repository),
      is_bare,
      path,
    }
  }

  pub(crate) async fn internal_find_commit(&self, target: Oid) -> anyhow::Result<Commit> {
    let repository = self.repository.lock().await;
    let commit = repository.find_commit(target.0)?;

    Ok(Commit::from(commit))
  }

  pub(crate) async fn internal_find_tree(&self, target: Oid) -> anyhow::Result<Tree> {
    let repository = self.repository.lock().await;
    let tree = repository.find_tree(target.0)?;

    Ok(Tree::from(tree))
  }

  pub(crate) async fn internal_create_commit(
    &self,
    update_ref: Option<String>,
    author: Signature,
    committer: Signature,
    message: String,
    tree: Tree,
    parents: Vec<Commit>,
  ) -> anyhow::Result<Oid> {
    let parents = parents
      .into_iter()
      .map(git2::Commit::from)
      .collect::<Vec<_>>();
    let parents_ptr = parents.iter().collect::<Vec<&git2::Commit>>();

    let repository = self.repository.lock().await;
    Ok(
      repository
        .commit(
          update_ref.as_deref(),
          &author.try_into()?,
          &committer.try_into()?,
          &message,
          &tree.into(),
          parents_ptr.as_slice(),
        )?
        .into(),
    )
  }

  async fn internal_fast_rebase(&self, onto_ref: String) -> anyhow::Result<()> {
    let committer = self.signature().await?;

    let repository = self.repository.lock().await;
    let reference = repository.find_reference(&onto_ref)?;
    let annotated_commit = repository.reference_to_annotated_commit(&reference)?;

    let current_branch_ref = repository.head()?;
    let current_branch = repository.reference_to_annotated_commit(&current_branch_ref)?;

    let mut rebase =
      repository.rebase(Some(&current_branch), None, Some(&annotated_commit), None)?;
    loop {
      let Some(op) = rebase.next() else {
        break;
      };
      let _ = op?;

      rebase.commit(None, &committer.clone().try_into()?, None)?;
    }

    rebase.finish(None)?;

    Ok(())
  }

  async fn internal_create_branch(
    &self,
    name: String,
    target: Oid,
    force: bool,
  ) -> anyhow::Result<crate::reference::Reference> {
    let repository = self.repository.lock().await;
    let commit = repository.find_commit(target.0)?;
    let branch = repository.branch(&name, &commit, force)?;

    Ok(crate::reference::Reference::new(branch.into_reference()))
  }

  async fn object_from_oid(&self, oid: git2::Oid) -> Result<crate::object::Object> {
    let repository = self.repository.lock().await;
    let object = repository
      .find_object(oid, None)
      .map_err(anyhow::Error::from)?;

    match object.kind() {
      Some(git2::ObjectType::Commit) | Some(git2::ObjectType::Tag) => {
        Ok(crate::object::Object::new(object))
      }
      _ => Err(anyhow::Error::msg("Invalid object type").into()),
    }
  }

  pub(crate) fn internal_fetch(
    &self,
    remote_name: String,
    fo: &mut git2::FetchOptions<'_>,
    prune: bool,
  ) -> anyhow::Result<()> {
    let repository = futures::executor::block_on(self.repository.lock());
    let mut remote = repository
      .find_remote(&remote_name)
      .or_else(|_| repository.remote_anonymous(&remote_name))
      .map_err(anyhow::Error::from)?;

    remote.download(&[] as &[&str], Some(fo))?;
    remote.disconnect()?;

    remote.update_tips(None, true, git2::AutotagOption::Unspecified, None)?;
    if prune {
      remote.prune(None)?;
    }

    Ok(())
  }

  pub(crate) async fn internal_checkout(&self, ref_name: &str) -> anyhow::Result<()> {
    let repository = self.repository.lock().await;
    let treeish = repository.revparse_single(ref_name)?;

    repository.checkout_tree(&treeish, Some(&mut CheckoutBuilder::new().force()))?;
    repository.set_head(ref_name)?;

    Ok(())
  }
}

#[napi]
#[derive(Default, Clone)]
pub struct InitOptions {
  pub(crate) bare: Option<bool>,
  pub(crate) initial_head: Option<String>,
}

#[napi]
impl InitOptions {
  #[napi(constructor)]
  pub fn new() -> InitOptions {
    Default::default()
  }

  #[napi]
  pub fn set_bare(&mut self, bare: bool) {
    let _ = self.bare.insert(bare);
  }

  #[napi]
  pub fn set_initial_head(&mut self, head: Option<String>) {
    self.initial_head = head;
  }
}

#[napi]
impl Repository {
  #[napi(ts_return_type = "Promise<Repository>")]
  pub fn init(
    path: String,
    options: Option<ClassInstance<InitOptions>>,
  ) -> AsyncTask<InitRepository> {
    AsyncTask::new(InitRepository::new(
      path,
      options.map(|o| o.as_ref().clone()).unwrap_or_default(),
    ))
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
  pub async fn namespace(&self) -> Option<String> {
    let repository = self.repository.lock().await;
    repository.namespace().map(ToString::to_string)
  }

  #[napi]
  pub fn is_bare(&self) -> bool {
    self.is_bare
  }

  #[napi]
  pub async fn is_empty(&self) -> napi::Result<bool> {
    let repository = self.repository.lock().await;
    Ok(repository.is_empty().map_err(anyhow::Error::from)?)
  }

  #[napi]
  pub fn path(&self) -> String {
    self.path.clone()
  }

  #[napi(ts_return_type = "Promise<void>")]
  pub async fn checkout(&self, ref_name: String) -> napi::Result<()> {
    self
      .internal_checkout(&ref_name)
      .await
      .map_err(|e| e.into())
  }

  #[napi(ts_return_type = "Promise<void>")]
  pub async fn fast_rebase(&self, onto_ref: String) -> napi::Result<()> {
    self
      .internal_fast_rebase(onto_ref)
      .await
      .map_err(|e| e.into())
  }

  #[napi(ts_return_type = "Promise<RepositoryState>")]
  pub async fn state(&self) -> RepositoryState {
    let repository = self.repository.lock().await;
    RepositoryState::from(repository.state())
  }

  #[napi(ts_return_type = "Promise<Commit>")]
  pub fn find_commit(
    &self,
    target: ClassInstance<Oid>,
    this: Reference<Repository>,
    env: Env,
  ) -> Result<JsObject> {
    let oid = *target;
    let (deferred, promise) = env.create_deferred()?;
    napi::tokio::spawn(async move {
      match this.internal_find_commit(oid).await {
        Ok(commit) => {
          deferred.resolve(|_| Ok(commit));
        }
        Err(e) => deferred.reject(e.into()),
      };
    });

    Ok(promise)
  }

  #[napi(ts_return_type = "Promise<Tree>")]
  pub fn find_tree(
    &self,
    target: ClassInstance<Oid>,
    this: Reference<Repository>,
    env: Env,
  ) -> Result<JsObject> {
    let oid = *target;
    let (deferred, promise) = env.create_deferred()?;
    napi::tokio::spawn(async move {
      match this.internal_find_tree(oid).await {
        Ok(tree) => {
          deferred.resolve(|_| Ok(tree));
        }
        Err(e) => deferred.reject(e.into()),
      };
    });

    Ok(promise)
  }

  #[napi(ts_return_type = "Promise<Remote>")]
  pub fn create_remote(
    &self,
    name: String,
    url: String,
    this: Reference<Repository>,
    env: Env,
  ) -> Result<JsObject> {
    let (deferred, promise) = env.create_deferred()?;
    napi::tokio::spawn(async move {
      let repository = this.repository.lock().await;
      let remote = repository.remote(&name, &url).map_err(anyhow::Error::from);

      let remote = match remote {
        Ok(r) => Remote::new(r),
        Err(e) => {
          deferred.reject(e.into());
          return;
        }
      };

      deferred.resolve(|_| Ok(remote));
    });

    Ok(promise)
  }

  #[napi(ts_return_type = "Promise<Remote>")]
  pub fn find_remote(
    &self,
    name: String,
    this: Reference<Repository>,
    env: Env,
  ) -> Result<JsObject> {
    let (deferred, promise) = env.create_deferred()?;
    napi::tokio::spawn(async move {
      let repository = this.repository.lock().await;
      let remote = repository.find_remote(&name).map_err(anyhow::Error::from);

      let remote = match remote {
        Ok(r) => Remote::new(r),
        Err(e) => {
          deferred.reject(e.into());
          return;
        }
      };

      deferred.resolve(|_| Ok(remote));
    });

    Ok(promise)
  }

  #[napi]
  pub fn create_branch(
    &self,
    name: String,
    commit: Either3<ClassInstance<Commit>, ClassInstance<Oid>, String>,
    force: bool,
    this: Reference<Repository>,
    env: Env,
  ) -> Result<JsObject> {
    let oid = match commit {
      Either3::A(commit) => commit.oid(),
      Either3::B(oid) => *oid,
      Either3::C(commit_sha) => Oid(git2::Oid::from_str(&commit_sha).map_err(anyhow::Error::from)?),
    };

    let (deferred, promise) = env.create_deferred()?;
    napi::tokio::spawn(async move {
      match this.internal_create_branch(name, oid, force).await {
        Ok(branch) => {
          deferred.resolve(|_| Ok(branch));
        }
        Err(e) => {
          deferred.reject(e.into());
        }
      };
    });

    Ok(promise)
  }

  #[napi(ts_return_type = "Promise<Commit>")]
  pub fn get_branch_commit(
    &self,
    name: Either<String, ClassInstance<crate::reference::Reference>>,
    this: Reference<Repository>,
  ) -> Result<AsyncTask<GetBranchCommit>> {
    let reference = match name {
      Either::A(name) => BranchNameRef::Name(name),
      Either::B(reference) => {
        let Some(oid) = reference.target() else {
          return Err(anyhow!("not a commit reference").into());
        };
        BranchNameRef::Reference(oid)
      }
    };

    Ok(AsyncTask::new(GetBranchCommit::new(this, reference)))
  }

  #[napi]
  pub async fn index(&self) -> Result<Index> {
    let repository = self.repository.lock().await;
    Ok(Index::from(
      repository.index().map_err(anyhow::Error::from)?,
    ))
  }

  #[napi]
  pub async fn signature(&self) -> Result<Signature> {
    let repository = self.repository.lock().await;
    Ok(Signature::from(
      repository.signature().map_err(anyhow::Error::from)?,
    ))
  }

  #[napi(ts_return_type = "Promise<Oid>")]
  #[allow(clippy::too_many_arguments)]
  pub fn create_commit(
    &self,
    update_ref: Option<String>,
    author: ClassInstance<Signature>,
    committer: ClassInstance<Signature>,
    message: String,
    tree: ClassInstance<Tree>,
    parents: Vec<ClassInstance<Commit>>,
    this: Reference<Repository>,
  ) -> AsyncTask<CreateCommit> {
    AsyncTask::new(CreateCommit::new(
      this, update_ref, author, committer, message, tree, parents,
    ))
  }

  #[napi(ts_return_type = "Promise<void>")]
  pub fn fetch(
    &self,
    options: Option<FetchOptions>,
    env: Env,
    this: Reference<Repository>,
  ) -> Result<AsyncTask<FetchRepository>> {
    let opts = options.unwrap_or_default();
    let fetch_opts = opts.into_fetch_opts(&env)?;

    Ok(AsyncTask::new(FetchRepository::new(this, fetch_opts)))
  }

  #[napi]
  pub async fn get_current_branch(&self) -> Result<crate::reference::Reference> {
    self.head().await
  }

  #[napi]
  pub async fn head(&self) -> Result<crate::reference::Reference> {
    let repository = self.repository.lock().await;
    let head = repository.head().map_err(anyhow::Error::from)?;

    Ok(crate::reference::Reference::new(head))
  }

  #[napi]
  pub async fn config(&self) -> Result<Config> {
    let repository = self.repository.lock().await;
    let config = repository.config().map_err(anyhow::Error::from)?;

    Ok(config.into())
  }

  #[napi(ts_return_type = "Promise<void>")]
  pub fn reset(
    &self,
    target: Either3<
      ClassInstance<Commit>,
      ClassInstance<crate::reference::Reference>,
      ClassInstance<Oid>,
    >,
    reset_type: Option<ResetType>,
    this: Reference<Repository>,
    env: Env,
  ) -> Result<JsObject> {
    let target = match target {
      Either3::A(commit) => Either::A(commit.clone()),
      Either3::B(reference) => {
        let oid = reference
          .target()
          .ok_or_else(|| anyhow::Error::msg("Cannot find reference target"))?;

        Either::B(oid)
      }
      Either3::C(oid) => Either::B(*oid),
    };

    let (deferred, promise) = env.create_deferred()?;
    napi::tokio::spawn(async move {
      let object = match target {
        Either::A(commit) => Ok(commit.as_object()),
        Either::B(oid) => this.object_from_oid(oid.0).await,
      };

      let object = match object {
        Ok(object) => object,
        Err(e) => {
          deferred.reject(e);
          return;
        }
      };

      let repository = this.repository.lock().await;
      let result = repository
        .reset(
          object.inner(),
          reset_type.unwrap_or(ResetType::Mixed).into(),
          None,
        )
        .map_err(anyhow::Error::from);

      match result {
        Ok(_) => {
          deferred.resolve(|_| Ok(()));
        }
        Err(e) => deferred.reject(e.into()),
      };
    });

    Ok(promise)
  }

  #[napi]
  pub async fn get_reference(&self, reference: String) -> Result<crate::reference::Reference> {
    let repository = self.repository.lock().await;
    let reference = repository
      .find_reference(&reference)
      .map_err(anyhow::Error::from)?;

    Ok(crate::reference::Reference::new(reference))
  }

  #[napi]
  pub async fn get_reference_names(
    &self,
    reference_type: Option<ReferenceType>,
  ) -> Result<Vec<String>> {
    let repository = self.repository.lock().await;
    let refs = repository
      .references()
      .map_err(anyhow::Error::from)?
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
      .collect();

    Ok(refs)
  }

  #[napi]
  pub async fn create_rev_walk(&self) -> napi::Result<Revwalk> {
    let repository = self.repository.lock().await;
    let rev_walk = repository.revwalk().map_err(anyhow::Error::from)?;

    Ok(rev_walk.into())
  }
}
