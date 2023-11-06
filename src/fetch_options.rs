use crate::credentials::Credentials;
use anyhow::Result;
use napi::bindgen_prelude::{ClassInstance, FromNapiValue};
use napi::{Env, JsFunction, JsUnknown, Ref};

#[napi(object)]
#[derive(Default)]
pub struct FetchOptions {
  pub remote: Option<String>,
  pub prune: Option<bool>,
  #[napi(ts_type = "(url: string, username?: string) => Credentials")]
  pub credentials_callback: Option<JsFunction>,
  pub skip_certificate_check: Option<bool>,
}

impl FetchOptions {
  pub fn into_fetch_opts(self, env: &Env) -> Result<FetchOpts> {
    let ret = FetchOpts {
      remote: self.remote,
      prune: self.prune,
      credentials_callback: self
        .credentials_callback
        .and_then(|cred_cb| env.create_reference(cred_cb).ok()),
      skip_certificate_check: self.skip_certificate_check,
    };

    Ok(ret)
  }
}

pub struct FetchOpts {
  pub remote: Option<String>,
  pub prune: Option<bool>,
  pub credentials_callback: Option<Ref<()>>,
  pub skip_certificate_check: Option<bool>,
}

impl FetchOpts {
  pub fn to_git_fetch_opts<'a>(&'a self, env: &'a Env) -> Result<git2::FetchOptions> {
    let mut cb = git2::RemoteCallbacks::default();

    {
      if let Some(cred_cb_ref) = self.credentials_callback.as_ref() {
        let cred_cb: JsFunction = env.get_reference_value(cred_cb_ref)?;
        cb.credentials(move |url, username, _| {
          (|| -> Result<ClassInstance<Credentials>, anyhow::Error> {
            let credentials = cred_cb.call::<JsUnknown>(
              None,
              &[
                env.create_string(url)?.into_unknown(),
                if let Some(username) = username {
                  env.create_string(username)?.into_unknown()
                } else {
                  env.get_undefined()?.into_unknown()
                },
              ],
            )?;

            Ok(ClassInstance::from_unknown(credentials)?)
          })()
          .and_then(|c| c.to_cred())
          .map_err(|err| git2::Error::from_str(&err.to_string()))
        });
      }
    }

    if self.skip_certificate_check.unwrap_or(false) {
      cb.certificate_check(|_cert, _domain| Ok(git2::CertificateCheckStatus::CertificateOk));
    }

    let mut fo = git2::FetchOptions::default();
    fo.remote_callbacks(cb);

    Ok(fo)
  }
}
