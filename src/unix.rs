// Expanded inside the `sync_impl` and `async_impl` submodules, both of
// which are feature-gated. With `--no-default-features` (nothing but
// the filesystem-stats API enabled), this macro has no caller, so
// silence the resulting unused-macro lint.
#[allow(unused_macros)]
macro_rules! lock_impl {
  ($file: ty) => {
    #[cfg(not(target_os = "wasi"))]
    pub fn lock_shared(file: &$file) -> std::io::Result<()> {
      flock(file, rustix::fs::FlockOperation::LockShared)
    }

    #[cfg(not(target_os = "wasi"))]
    pub fn lock(file: &$file) -> std::io::Result<()> {
      flock(file, rustix::fs::FlockOperation::LockExclusive)
    }

    #[cfg(not(target_os = "wasi"))]
    pub fn try_lock_shared(file: &$file) -> std::result::Result<(), $crate::TryLockError> {
      try_flock(file, rustix::fs::FlockOperation::NonBlockingLockShared)
    }

    #[cfg(not(target_os = "wasi"))]
    pub fn try_lock(file: &$file) -> std::result::Result<(), $crate::TryLockError> {
      try_flock(file, rustix::fs::FlockOperation::NonBlockingLockExclusive)
    }

    #[cfg(not(target_os = "wasi"))]
    pub fn unlock(file: &$file) -> std::io::Result<()> {
      flock(file, rustix::fs::FlockOperation::Unlock)
    }

    // `BorrowedFd::borrow_raw` is used (instead of `AsFd::as_fd`)
    // because half of the supported file types -- `async_std::fs::File`,
    // `smol::fs::File`, `fs_err2::tokio::File`, and `fs_err3::tokio::File`
    // -- only expose `AsRawFd` upstream. Routing everything through
    // `as_raw_fd()` keeps one implementation instead of splitting the
    // macro into safe/unsafe variants. The `BorrowedFd` never outlives
    // the `&$file` reference, so the bounded lifetime holds even
    // though the conversion itself is `unsafe`.
    #[cfg(not(target_os = "wasi"))]
    fn flock(file: &$file, flag: rustix::fs::FlockOperation) -> std::io::Result<()> {
      let borrowed_fd = unsafe { rustix::fd::BorrowedFd::borrow_raw(file.as_raw_fd()) };

      rustix::fs::flock(borrowed_fd, flag)
        .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))
    }

    #[cfg(not(target_os = "wasi"))]
    fn try_flock(
      file: &$file,
      flag: rustix::fs::FlockOperation,
    ) -> std::result::Result<(), $crate::TryLockError> {
      let borrowed_fd = unsafe { rustix::fd::BorrowedFd::borrow_raw(file.as_raw_fd()) };

      rustix::fs::flock(borrowed_fd, flag)
        .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()).into())
    }
  };
}

#[cfg(any(
  feature = "smol",
  feature = "async-std",
  feature = "tokio",
  feature = "fs-err2-tokio",
  feature = "fs-err3-tokio",
))]
pub(crate) mod async_impl;
#[cfg(any(feature = "sync", feature = "fs-err2", feature = "fs-err3"))]
pub(crate) mod sync_impl;

use crate::FsStats;

use std::io::Result;
use std::path::Path;

pub fn statvfs(path: impl AsRef<Path>) -> Result<FsStats> {
  match rustix::fs::statvfs(path.as_ref()) {
    Ok(stat) => Ok(FsStats {
      free_space: stat.f_frsize.saturating_mul(stat.f_bfree),
      available_space: stat.f_frsize.saturating_mul(stat.f_bavail),
      total_space: stat.f_frsize.saturating_mul(stat.f_blocks),
      allocation_granularity: stat.f_frsize,
    }),
    Err(e) => Err(std::io::Error::from_raw_os_error(e.raw_os_error())),
  }
}
