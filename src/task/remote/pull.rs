use crate::remote::Remote;
use napi::bindgen_prelude::Reference;
use napi::{Env, Ref, Task};

pub struct PullRemote {
  remote: Reference<Remote>,
  ref_specs: Vec<String>,
  credentials_callback: Option<Ref<()>>,
}

impl PullRemote {
  pub fn new(
    remote: Reference<Remote>,
    ref_specs: Vec<String>,
    credentials_callback: Option<Ref<()>>,
  ) -> Self {
    Self {
      remote,
      ref_specs,
      credentials_callback,
    }
  }
}

#[napi]
impl Task for PullRemote {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> napi::Result<Self::Output> {
    Ok(())
  }

  fn resolve(&mut self, env: Env, _: Self::Output) -> napi::Result<Self::JsValue> {
    let cb = Remote::prepare_remote_callbacks(
      self
        .credentials_callback
        .as_ref()
        .and_then(|c| env.get_reference_value(c).ok()),
      &env,
    )?;
    let ut_cb = Remote::prepare_remote_callbacks(
      self
        .credentials_callback
        .as_ref()
        .and_then(|c| env.get_reference_value(c).ok()),
      &env,
    )?;

    self
      .remote
      .internal_pull(self.ref_specs.as_slice(), cb, ut_cb)?;

    Ok(())
  }
}
