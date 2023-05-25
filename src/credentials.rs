use anyhow::Result;
use git2::Cred;

#[derive(Clone)]
enum CredentialType {
  Default,
  UsernamePassword(String, String),
  SshKeyFromMemory(String, Option<String>, String, Option<String>),
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

  #[napi(factory)]
  pub fn ssh_key_from_memory(
    username: String,
    public_key: Option<String>,
    private_key: String,
    passphrase: Option<String>,
  ) -> Self {
    Self(CredentialType::SshKeyFromMemory(
      username,
      public_key,
      private_key,
      passphrase,
    ))
  }

  pub fn to_cred(&self) -> Result<Cred> {
    Ok(match &self.0 {
      CredentialType::Default => Cred::default()?,
      CredentialType::UsernamePassword(u, p) => Cred::userpass_plaintext(u, p)?,
      CredentialType::SshKeyFromMemory(u, pub_key, priv_key, passphrase) => {
        Cred::ssh_key_from_memory(
          u,
          pub_key.as_ref().map(AsRef::as_ref),
          priv_key,
          passphrase.as_ref().map(AsRef::as_ref),
        )?
      }
    })
  }
}
