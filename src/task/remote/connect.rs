use crate::remote::Remote;
use crate::Direction;
use napi::bindgen_prelude::Reference;
use napi::{Env, Ref, Task};

pub struct ConnectRemote {
  remote: Reference<Remote>,
  direction: Direction,
  credentials_callback: Option<Ref<()>>,
}

impl ConnectRemote {
  pub fn new(
    remote: Reference<Remote>,
    direction: Direction,
    credentials_callback: Option<Ref<()>>,
  ) -> Self {
    Self {
      remote,
      direction,
      credentials_callback,
    }
  }
}

#[napi]
impl Task for ConnectRemote {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> napi::Result<Self::Output> {
    Ok(())
  }

  fn resolve(&mut self, env: Env, _: Self::Output) -> napi::Result<Self::JsValue> {
    let dir = match self.direction {
      Direction::Fetch => git2::Direction::Fetch,
      Direction::Push => git2::Direction::Push,
    };

    let cb = Remote::prepare_remote_callbacks(
      self
        .credentials_callback
        .as_ref()
        .and_then(|c| env.get_reference_value(c).ok()),
      &env,
    )?;
    self.remote.internal_connect(dir, cb)?;

    Ok(())
  }
}
