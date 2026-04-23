//! Extended utilities for working with files and filesystems in Rust.
#![doc(html_root_url = "https://docs.rs/fs4/1.0.1")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![allow(unexpected_cfgs, unstable_name_collisions)]
// The `cfg_<feature>!` macros below are only invoked by the Unix /
// Windows `file_ext` backends, so on targets where neither `cfg(unix)`
// nor `cfg(windows)` matches (e.g. `wasm32-wasi*`) they appear
// unused. The crate still compiles there, just without the
// filesystem extension traits, so silence the lint globally rather
// than shadowing the macro definitions.
#![cfg_attr(not(any(unix, windows)), allow(unused_macros))]

#[cfg(windows)]
extern crate windows_sys;

macro_rules! cfg_async_std {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "async-std")]
            #[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
            $item
        )*
    }
}

macro_rules! cfg_fs_err2 {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "fs-err2")]
            #[cfg_attr(docsrs, doc(cfg(feature = "fs-err2")))]
            $item
        )*
    }
}

macro_rules! cfg_fs_err2_tokio {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "fs-err2-tokio")]
            #[cfg_attr(docsrs, doc(cfg(feature = "fs-err2-tokio")))]
            $item
        )*
    }
}

macro_rules! cfg_fs_err3 {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "fs-err3")]
            #[cfg_attr(docsrs, doc(cfg(feature = "fs-err3")))]
            $item
        )*
    }
}

macro_rules! cfg_fs_err3_tokio {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "fs-err3-tokio")]
            #[cfg_attr(docsrs, doc(cfg(feature = "fs-err3-tokio")))]
            $item
        )*
    }
}

macro_rules! cfg_smol {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "smol")]
            #[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
            $item
        )*
    }
}

macro_rules! cfg_tokio {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "tokio")]
            #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
            $item
        )*
    }
}

macro_rules! cfg_sync {
  ($($item:item)*) => {
      $(
          #[cfg(feature = "sync")]
          #[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
          $item
      )*
  }
}

macro_rules! cfg_async {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "smol",
                feature = "async-std",
                feature = "tokio",
                feature = "fs-err2-tokio",
                feature = "fs-err3-tokio",
            ))]
            #[cfg_attr(docsrs, doc(cfg(any(
                feature = "smol",
                feature = "async-std",
                feature = "tokio",
                feature = "fs-err2-tokio",
                feature = "fs-err3-tokio",
            ))))]
            $item
        )*
    }
}

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix as sys;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows as sys;

// The file-extension traits (`FileExt`, `AsyncFileExt`) and the stats
// API are only implementable on targets with a real `sys` backend.
// Anywhere else (notably `wasm32-wasi*`, where `target_family = "wasm"`
// so neither `cfg(unix)` nor `cfg(windows)` matches and rustix does
// not expose `statvfs` / `flock` / `fallocate`) the crate compiles
// down to just the shared data types below.
#[cfg(any(unix, windows))]
mod file_ext;

#[cfg(all(feature = "sync", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
pub use crate::file_ext::sync_impl::std_impl::FileExt;

#[cfg(all(feature = "fs-err2", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "fs-err2")))]
pub mod fs_err2 {
  pub use crate::file_ext::sync_impl::fs_err2_impl::FileExt;
}

#[cfg(all(feature = "fs-err3", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "fs-err3")))]
pub mod fs_err3 {
  pub use crate::file_ext::sync_impl::fs_err3_impl::FileExt;
}

#[cfg(all(feature = "async-std", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
pub mod async_std {
  pub use crate::file_ext::async_impl::async_std_impl::AsyncFileExt;
}

#[cfg(all(feature = "fs-err2-tokio", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "fs-err2-tokio")))]
pub mod fs_err2_tokio {
  pub use crate::file_ext::async_impl::fs_err2_tokio_impl::AsyncFileExt;
}

#[cfg(all(feature = "fs-err3-tokio", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "fs-err3-tokio")))]
pub mod fs_err3_tokio {
  pub use crate::file_ext::async_impl::fs_err3_tokio_impl::AsyncFileExt;
}

#[cfg(all(feature = "smol", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
pub mod smol {
  pub use crate::file_ext::async_impl::smol_impl::AsyncFileExt;
}

#[cfg(all(feature = "tokio", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio {
  pub use crate::file_ext::async_impl::tokio_impl::AsyncFileExt;
}

mod fs_stats;
pub use fs_stats::FsStats;

mod try_lock_error;
pub use try_lock_error::TryLockError;

#[cfg(any(unix, windows))]
use std::io::Result;
#[cfg(any(unix, windows))]
use std::path::Path;

/// Get the stats of the file system containing the provided path.
#[cfg(any(unix, windows))]
pub fn statvfs<P>(path: P) -> Result<FsStats>
where
  P: AsRef<Path>,
{
  sys::statvfs(path.as_ref())
}

/// Returns the number of free bytes in the file system containing the provided
/// path.
#[cfg(any(unix, windows))]
pub fn free_space<P>(path: P) -> Result<u64>
where
  P: AsRef<Path>,
{
  statvfs(path).map(|stat| stat.free_space)
}

/// Returns the available space in bytes to non-privileged users in the file
/// system containing the provided path.
#[cfg(any(unix, windows))]
pub fn available_space<P>(path: P) -> Result<u64>
where
  P: AsRef<Path>,
{
  statvfs(path).map(|stat| stat.available_space)
}

/// Returns the total space in bytes in the file system containing the provided
/// path.
#[cfg(any(unix, windows))]
pub fn total_space<P>(path: P) -> Result<u64>
where
  P: AsRef<Path>,
{
  statvfs(path).map(|stat| stat.total_space)
}

/// Returns the filesystem's disk space allocation granularity in bytes.
/// The provided path may be for any file in the filesystem.
///
/// On Posix, this is equivalent to the filesystem's block size.
/// On Windows, this is equivalent to the filesystem's cluster size.
#[cfg(any(unix, windows))]
pub fn allocation_granularity<P>(path: P) -> Result<u64>
where
  P: AsRef<Path>,
{
  statvfs(path).map(|stat| stat.allocation_granularity)
}

#[cfg(all(test, any(unix, windows)))]
mod tests {
  //! The `free_space` / `available_space` / `total_space` helpers
  //! each forward to `statvfs(...).map(|s| s.<field>)`. The
  //! `FsStats` getter tests in `fs_stats.rs` cover the field
  //! accessors; these tests cover the top-level forwarders (which
  //! were previously uncovered in CI per Codecov).
  //!
  //! Assertions are intentionally loose: we don't compare the three
  //! numbers across separate `statvfs` calls because that races
  //! with concurrent filesystem activity (other tests, the OS,
  //! etc.). Proving the call returned `Ok` with a plausible value
  //! is enough to exercise the forwarding path.
  extern crate tempfile;

  use super::*;

  fn tempdir() -> tempfile::TempDir {
    tempfile::TempDir::with_prefix("fs4").unwrap()
  }

  #[test]
  fn free_space_returns_ok() {
    let dir = tempdir();
    let free = free_space(dir.path()).unwrap();
    let total = total_space(dir.path()).unwrap();
    assert!(
      free <= total,
      "free_space ({free}) must not exceed total_space ({total})",
    );
  }

  #[test]
  fn available_space_returns_ok() {
    let dir = tempdir();
    let available = available_space(dir.path()).unwrap();
    let total = total_space(dir.path()).unwrap();
    assert!(
      available <= total,
      "available_space ({available}) must not exceed total_space ({total})",
    );
  }

  #[test]
  fn total_space_is_non_zero() {
    let dir = tempdir();
    assert!(
      total_space(dir.path()).unwrap() > 0,
      "total_space on a tempdir's volume should be non-zero",
    );
  }

  /// POSIX `statvfs` returns `ENOENT` for a path that doesn't
  /// exist, which is how we exercise the error-propagation branch
  /// of the three forwarders. Windows has different semantics:
  /// `GetVolumePathNameW` resolves any syntactically valid path to
  /// its volume root regardless of whether the path itself exists,
  /// so `statvfs(missing)` returns `Ok` on that platform.
  #[cfg(unix)]
  #[test]
  fn missing_path_errors() {
    let dir = tempdir();
    let missing = dir.path().join("definitely-does-not-exist");
    assert!(free_space(&missing).is_err());
    assert!(available_space(&missing).is_err());
    assert!(total_space(&missing).is_err());
  }
}
