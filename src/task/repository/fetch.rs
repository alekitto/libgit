use crate::fetch_options::FetchOpts;
use crate::repository::Repository;
use anyhow::Result;
use napi::bindgen_prelude::Reference;
use napi::{Env, Task};

pub struct FetchRepository {
  repository: Reference<Repository>,
  fetch_options: FetchOpts,
}

impl FetchRepository {
  pub fn new(repository: Reference<Repository>, fetch_options: FetchOpts) -> Self {
    Self {
      repository,
      fetch_options,
    }
  }
}

#[napi]
impl Task for FetchRepository {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> napi::Result<Self::Output> {
    Ok(())
  }

  fn resolve(&mut self, env: Env, _: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(fetch(&self.repository, &self.fetch_options, &env)?)
  }
}

fn fetch(repository: &Repository, fetch_options: &FetchOpts, env: &Env) -> Result<()> {
  let remote_name = fetch_options
    .remote
    .clone()
    .unwrap_or_else(|| "origin".to_string());

  let mut fo = fetch_options.to_git_fetch_opts(&env)?;
  let mut remote = repository
    .repository
    .find_remote(&remote_name)
    .or_else(|_| repository.repository.remote_anonymous(&remote_name))?;
  remote.download(&[] as &[&str], Some(&mut fo))?;
  remote.disconnect()?;

  remote.update_tips(None, true, git2::AutotagOption::Unspecified, None)?;
  if fetch_options.prune.unwrap_or(false) {
    remote.prune(None)?;
  }

  Ok(())
}
