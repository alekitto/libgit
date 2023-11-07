#[napi]
pub struct Config {
  inner: git2::Config,
}

impl From<git2::Config> for Config {
  fn from(value: git2::Config) -> Self {
    Self { inner: value }
  }
}

#[napi]
impl Config {
  #[napi]
  pub fn set_str(&mut self, name: String, value: String) -> napi::Result<()> {
    self
      .inner
      .set_str(&name, &value)
      .map_err(anyhow::Error::from)?;

    Ok(())
  }

  #[napi]
  pub fn set_bool(&mut self, name: String, value: bool) -> napi::Result<()> {
    self
      .inner
      .set_bool(&name, value)
      .map_err(anyhow::Error::from)?;

    Ok(())
  }

  #[napi]
  pub fn set_i64(&mut self, name: String, value: i64) -> napi::Result<()> {
    self
      .inner
      .set_i64(&name, value)
      .map_err(anyhow::Error::from)?;

    Ok(())
  }
}
