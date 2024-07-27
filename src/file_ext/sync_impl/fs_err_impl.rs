#[cfg(unix)]
use crate::unix::sync_impl::fs_err_impl as sys;
#[cfg(windows)]
use crate::windows::sync_impl::fs_err_impl as sys;
use fs_err::File;

file_ext!(File, "fs_err::File");

test_mod! {
  use fs_err as fs;
}
