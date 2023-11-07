use crate::object::Oid;
use napi::bindgen_prelude::*;
use napi::tokio::sync::Mutex;
use std::path::Path;

#[napi]
pub struct Index {
  inner: Mutex<git2::Index>,
}

impl From<git2::Index> for Index {
  fn from(value: git2::Index) -> Self {
    Self {
      inner: Mutex::new(value),
    }
  }
}

#[napi]
impl Index {
  #[napi]
  pub async fn add_path(&self, path: String) -> Result<()> {
    let mut index = self.inner.lock().await;
    Ok(
      index
        .add_path(Path::new(&path))
        .map_err(anyhow::Error::from)?,
    )
  }

  #[napi]
  pub async fn write_tree(&self) -> Result<Oid> {
    let mut index = self.inner.lock().await;
    Ok(index.write_tree().map(Oid).map_err(anyhow::Error::from)?)
  }
}
