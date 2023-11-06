use crate::credentials::Credentials;
use crate::object::Oid;
use crate::Direction;
use anyhow::Result;
use napi::bindgen_prelude::*;
use napi::JsUnknown;

#[napi]
pub struct Remote {
  inner: git2::Remote<'static>,
}

impl Remote {
  pub(crate) fn new(remote: git2::Remote<'_>) -> Self {
    Self {
      inner: unsafe { std::mem::transmute(remote) },
    }
  }
}

#[napi]
pub struct RemoteHead {
  name: String,
  oid: Oid,
  loid: Option<Oid>,
}

impl Remote {
  fn prepare_remote_callbacks(
    credentials_callback: Option<JsFunction>,
    env: &Env,
  ) -> Result<git2::RemoteCallbacks> {
    let mut cb = git2::RemoteCallbacks::default();
    if let Some(cred_cb) = credentials_callback {
      cb.credentials(move |url, username, _| {
        cred_cb
          .call::<JsUnknown>(
            None,
            &[
              env
                .create_string(url)
                .map_err(|e| git2::Error::from_str(&e.reason))?
                .into_unknown(),
              if let Some(username) = username {
                env
                  .create_string(username)
                  .map_err(|e| git2::Error::from_str(&e.reason))?
                  .into_unknown()
              } else {
                env
                  .get_undefined()
                  .map_err(|e| git2::Error::from_str(&e.reason))?
                  .into_unknown()
              },
            ],
          )
          .map_err(|e| git2::Error::from_str(&e.reason))
          .and_then(|c| {
            ClassInstance::from_unknown(c).map_err(|e| git2::Error::from_str(&e.reason))
          })
          .and_then(|c: ClassInstance<Credentials>| {
            c.to_cred()
              .map_err(|e| git2::Error::from_str(&e.to_string()))
          })
      });
    }

    Ok(cb)
  }
}

#[napi]
impl Remote {
  #[napi]
  pub fn connect(
    &mut self,
    direction: Direction,
    credentials_callback: Option<JsFunction>,
    env: Env,
  ) -> Result<()> {
    let dir = match direction {
      Direction::Fetch => git2::Direction::Fetch,
      Direction::Push => git2::Direction::Push,
    };

    let cb = Self::prepare_remote_callbacks(credentials_callback, &env)?;
    self.inner.connect_auth(dir, Some(cb), None)?;

    Ok(())
  }

  #[napi]
  pub fn disconnect(&mut self) -> Result<()> {
    Ok(self.inner.disconnect()?)
  }

  #[napi]
  pub fn reference_list(&self) -> Result<Vec<RemoteHead>> {
    let list = self.inner.list().map_err(anyhow::Error::from)?;
    Ok(
      list
        .iter()
        .map(|head| RemoteHead {
          name: head.name().to_string(),
          oid: head.oid().into(),
          loid: if head.is_local() {
            Some(head.loid().into())
          } else {
            None
          },
        })
        .collect(),
    )
  }
}

#[napi]
impl RemoteHead {
  #[napi]
  pub fn name(&self) -> String {
    self.name.clone()
  }

  #[napi]
  pub fn oid(&self) -> Oid {
    self.oid
  }

  #[napi]
  pub fn is_local(&self) -> bool {
    self.loid.is_some()
  }

  #[napi]
  pub fn local_oid(&self) -> Option<Oid> {
    self.loid
  }
}
