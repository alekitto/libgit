use crate::object::Oid;
use crate::Sort;
use napi::bindgen_prelude::*;
use napi::tokio::sync::Mutex;
use napi::JsObject;

#[napi]
pub struct Revwalk {
  inner: Mutex<git2::Revwalk<'static>>,
}

impl From<git2::Revwalk<'_>> for Revwalk {
  fn from(value: git2::Revwalk<'_>) -> Self {
    Self::new(value)
  }
}

impl Revwalk {
  pub(crate) fn new(revwalk: git2::Revwalk<'_>) -> Self {
    Self {
      inner: Mutex::new(unsafe { std::mem::transmute(revwalk) }),
    }
  }
}

#[napi]
impl Revwalk {
  #[napi]
  pub fn push(
    &self,
    oid: ClassInstance<Oid>,
    this: Reference<Revwalk>,
    env: Env,
  ) -> Result<JsObject> {
    let oid = oid.0;

    let (deferred, promise) = env.create_deferred()?;
    napi::tokio::spawn(async move {
      let mut inner = this.inner.lock().await;
      match inner.push(oid).map_err(anyhow::Error::from) {
        Ok(()) => deferred.resolve(|_| Ok(())),
        Err(e) => deferred.reject(e.into()),
      }
    });

    Ok(promise)
  }

  #[napi]
  pub fn push_range(&self, range: String, this: Reference<Revwalk>, env: Env) -> Result<JsObject> {
    let (deferred, promise) = env.create_deferred()?;
    napi::tokio::spawn(async move {
      let mut inner = this.inner.lock().await;
      match inner.push_range(&range).map_err(anyhow::Error::from) {
        Ok(()) => deferred.resolve(|_| Ok(())),
        Err(e) => deferred.reject(e.into()),
      }
    });

    Ok(promise)
  }

  #[napi]
  pub async fn next(&self) -> Result<Option<Oid>> {
    let mut inner = self.inner.lock().await;
    let o = inner.next();
    match o {
      Some(Ok(r)) => Ok(Some(Oid(r))),
      Some(Err(e)) => Err(anyhow::Error::from(e).into()),
      None => Ok(None),
    }
  }

  #[napi]
  pub async fn reset(&self) -> Result<()> {
    let mut inner = self.inner.lock().await;
    inner.reset().map_err(anyhow::Error::from)?;

    Ok(())
  }

  #[napi]
  pub async fn sort(&self, sorts: Vec<Sort>) -> Result<()> {
    let mut sort = git2::Sort::NONE;

    for s in sorts {
      sort |= s.into();
    }

    let mut inner = self.inner.lock().await;
    inner.set_sorting(sort).map_err(anyhow::Error::from)?;

    Ok(())
  }
}
