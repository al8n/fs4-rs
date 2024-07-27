#[cfg(unix)]
use crate::unix::async_impl::fs_err_tokio_impl as sys;
#[cfg(windows)]
use crate::windows::async_impl::fs_err_tokio_impl as sys;
use fs_err::tokio::File;

async_file_ext!(File, "fs_err::tokio::File");

test_mod! {
  tokio::test,
  use crate::fs_err_tokio::AsyncFileExt;
  use fs_err::tokio as fs;
}
