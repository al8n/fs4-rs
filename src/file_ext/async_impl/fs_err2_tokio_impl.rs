#[cfg(unix)]
use crate::unix::async_impl::fs_err2_tokio_impl as sys;
#[cfg(windows)]
use crate::windows::async_impl::fs_err2_tokio_impl as sys;
use fs_err2::tokio::File;

async_file_ext!(File, "fs_err::tokio::File");

test_mod! {
  tokio::test,
  use fs_err2::tokio as fs;
}
