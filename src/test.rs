use std::path::{Path, PathBuf};
use std::{io, ptr};

#[cfg(windows)]
pub fn realpath(original: &Path) -> io::Result<PathBuf> {
  Ok(original.to_path_buf())
}
#[cfg(unix)]
pub fn realpath(original: &Path) -> io::Result<PathBuf> {
  use libc::c_char;
  use std::ffi::{CStr, CString, OsString};
  use std::os::unix::prelude::*;
  extern "C" {
    fn realpath(name: *const c_char, resolved: *mut c_char) -> *mut c_char;
  }
  unsafe {
    let cstr = CString::new(original.as_os_str().as_bytes())?;
    let ptr = realpath(cstr.as_ptr(), ptr::null_mut());
    if ptr.is_null() {
      return Err(io::Error::last_os_error());
    }
    let bytes = CStr::from_ptr(ptr).to_bytes().to_vec();
    libc::free(ptr as *mut _);
    Ok(PathBuf::from(OsString::from_vec(bytes)))
  }
}
