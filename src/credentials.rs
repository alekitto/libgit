use anyhow::Result;
use git2::Cred;

#[derive(Clone)]
enum CredentialType {
  Default,
  UsernamePassword(String, String),
}

#[napi]
#[derive(Clone)]
pub struct Credentials(CredentialType);

#[napi]
impl Credentials {
  #[napi(factory)]
  pub fn default() -> Self {
    Self(CredentialType::Default)
  }

  #[napi(factory)]
  pub fn username_and_password(username: String, password: String) -> Self {
    Self(CredentialType::UsernamePassword(username, password))
  }

  pub fn to_cred(&self) -> Result<Cred> {
    Ok(match &self.0 {
      CredentialType::Default => Cred::default()?,
      CredentialType::UsernamePassword(u, p) => Cred::userpass_plaintext(u, p)?,
    })
  }
}
