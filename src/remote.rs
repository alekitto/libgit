use crate::credentials::Credentials;
use crate::object::Oid;
use crate::task::{ConnectRemote, PushRemote};
use crate::Direction;
use anyhow::Result;
use git2::{PushOptions, RemoteCallbacks, RemoteConnection};
use napi::bindgen_prelude::*;
use napi::tokio::sync::Mutex;
use napi::JsUnknown;
use std::pin::Pin;

pub struct RemoteConn(RemoteConnection<'static, 'static, 'static>);
unsafe impl Send for RemoteConn {}

#[napi]
pub struct Remote {
  inner: Mutex<git2::Remote<'static>>,
  connection: Mutex<Pin<Box<Option<RemoteConn>>>>,
}

impl Remote {
  pub(crate) fn new(remote: git2::Remote<'_>) -> Self {
    Self {
      inner: Mutex::new(unsafe { std::mem::transmute(remote) }),
      connection: Mutex::new(Box::pin(None)),
    }
  }

  pub(crate) fn internal_connect(
    &self,
    direction: git2::Direction,
    remote_callbacks: RemoteCallbacks,
  ) -> Result<()> {
    let mut remote = futures::executor::block_on(self.inner.lock());
    let connection = remote.connect_auth(direction, Some(remote_callbacks), None)?;

    let mut conn = futures::executor::block_on(self.connection.lock());
    let _ = conn.insert(RemoteConn(unsafe { std::mem::transmute(connection) }));

    Ok(())
  }

  pub(crate) fn internal_push(
    &self,
    ref_specs: &[String],
    remote_callbacks: RemoteCallbacks,
  ) -> Result<()> {
    let mut po = PushOptions::default();
    po.remote_callbacks(remote_callbacks);

    let mut remote = futures::executor::block_on(self.inner.lock());
    remote.push(ref_specs, Some(&mut po))?;

    Ok(())
  }
}

#[napi]
pub struct RemoteHead {
  name: String,
  oid: Oid,
  loid: Option<Oid>,
}

impl Remote {
  pub(crate) fn prepare_remote_callbacks(
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
  #[napi(ts_return_type = "Promise<void>")]
  pub fn connect(
    &self,
    direction: Direction,
    #[napi(ts_arg_type = "(url: string, username?: string) => Credentials")] credentials_callback: Option<JsFunction>,
    env: Env,
    this: Reference<Remote>,
  ) -> napi::Result<AsyncTask<ConnectRemote>> {
    let cb_ref = if let Some(f) = credentials_callback {
      Some(env.create_reference(f)?)
    } else {
      None
    };

    Ok(AsyncTask::new(ConnectRemote::new(this, direction, cb_ref)))
  }

  #[napi]
  pub async fn disconnect(&self) -> napi::Result<()> {
    let mut inner = self.connection.lock().await;
    drop(inner.take());

    Ok(())
  }

  #[napi]
  pub async fn reference_list(&self) -> napi::Result<Vec<RemoteHead>> {
    let inner = self.inner.lock().await;
    let list = inner.list().map_err(anyhow::Error::from)?;
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

  #[napi(ts_return_type = "Promise<void>")]
  pub fn push(
    &self,
    ref_specs: Vec<String>,
    #[napi(ts_arg_type = "(url: string, username?: string) => Credentials")] credentials_callback: Option<JsFunction>,
    env: Env,
    this: Reference<Remote>,
  ) -> napi::Result<AsyncTask<PushRemote>> {
    let cb_ref = if let Some(f) = credentials_callback {
      Some(env.create_reference(f)?)
    } else {
      None
    };

    Ok(AsyncTask::new(PushRemote::new(this, ref_specs, cb_ref)))
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
