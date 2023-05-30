use crate::fetch_options::FetchOpts;
use crate::repository::Repository;
use anyhow::Result;
use git2::build::RepoBuilder;
use napi::{Env, Task};
use std::path::Path;

pub struct CloneRepository {
  url: String,
  directory: String,
  recursive: bool,
  fetch_options: FetchOpts,
}

impl CloneRepository {
  pub fn new<P: AsRef<Path>>(
    url: &str,
    directory: P,
    recursive: bool,
    fetch_options: FetchOpts,
  ) -> Self {
    Self {
      url: url.to_string(),
      directory: directory.as_ref().to_string_lossy().to_string(),
      recursive,
      fetch_options,
    }
  }
}

impl Task for CloneRepository {
  type Output = ();
  type JsValue = Repository;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    Ok(())
  }

  fn resolve(&mut self, env: Env, _: Self::Output) -> napi::Result<Self::JsValue> {
    let repository = clone(
      &self.url,
      &self.directory,
      self.recursive,
      &self.fetch_options,
      &env,
    )?;

    Ok(Repository::from(repository))
  }
}

fn clone<P: AsRef<Path>>(
  url: &str,
  directory: P,
  recursive: bool,
  fetch_options: &FetchOpts,
  env: &Env,
) -> Result<git2::Repository> {
  let fo = fetch_options.to_git_fetch_opts(env)?;
  let repository = RepoBuilder::new()
    .fetch_options(fo)
    .clone(url, directory.as_ref())?;

  if recursive {
    for mut submodule in repository.submodules()? {
      submodule.update(true, None)?;
    }
  }

  Ok(repository)
}
