#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
// The `cfg_<feature>!` macros below are only invoked inside
// feature-gated modules -- every call site is itself behind
// `#[cfg(feature = "...")]` or inside the Unix/Windows backend
// trees. With `--no-default-features` (or on targets where neither
// `cfg(unix)` nor `cfg(windows)` matches, e.g. `wasm32-wasi*`), all
// call sites compile out, so the macros appear unused. Silence the
// lint at the crate level rather than shadowing each definition.
#![allow(unexpected_cfgs, unstable_name_collisions, unused_macros)]

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

#[cfg(all(feature = "fs-err2", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "fs-err2")))]
pub mod fs_err2 {
  pub use crate::FileExt;
}

#[cfg(all(feature = "fs-err3", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "fs-err3")))]
pub mod fs_err3 {
  pub use crate::FileExt;
}

#[cfg(all(feature = "async-std", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
pub mod async_std {
  pub use crate::{AsyncFileExt, DynAsyncFileExt};
}

#[cfg(all(feature = "fs-err2-tokio", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "fs-err2-tokio")))]
pub mod fs_err2_tokio {
  pub use crate::{AsyncFileExt, DynAsyncFileExt};
}

#[cfg(all(feature = "fs-err3-tokio", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "fs-err3-tokio")))]
pub mod fs_err3_tokio {
  pub use crate::{AsyncFileExt, DynAsyncFileExt};
}

#[cfg(all(feature = "smol", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
pub mod smol {
  pub use crate::{AsyncFileExt, DynAsyncFileExt};
}

#[cfg(all(feature = "tokio", any(unix, windows)))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio {
  pub use crate::{AsyncFileExt, DynAsyncFileExt};
}

mod fs_stats;
pub use fs_stats::FsStats;

mod try_lock_error;
pub use try_lock_error::TryLockError;

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

mod sealed {
  pub trait Sealed {}

  impl<F: Sealed + ?Sized> Sealed for &F {}
}

/// Extension trait for file which provides allocation and locking methods.
///
/// This trait is sealed and cannot be implemented for types outside of `fs4`.
///
/// ## Notes on File Locks
///
/// This library provides whole-file locks in both shared (read) and exclusive
/// (read-write) varieties.
///
/// File locks are a cross-platform hazard since the file lock APIs exposed by
/// operating system kernels vary in subtle and not-so-subtle ways.
///
/// The API exposed by this library can be safely used across platforms as long
/// as the following rules are followed:
///
///   * Multiple locks should not be created on an individual `File` instance
///     concurrently.
///   * Duplicated files should not be locked without great care.
///   * Files to be locked should be opened with at least read or write
///     permissions.
///   * File locks may only be relied upon to be advisory.
///
/// File locks are released automatically when the file handle is closed (for
/// example when the owning `File` is dropped), so calling [`FileExt::unlock`]
/// explicitly is optional.
///
/// File locks are implemented with
/// [`flock(2)`](http://man7.org/linux/man-pages/man2/flock.2.html) on Unix and
/// [`LockFileEx`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-lockfileex)
/// on Windows.
pub trait FileExt: sealed::Sealed {
  /// Returns the amount of physical space allocated for a file.
  fn allocated_size(&self) -> Result<u64>;

  /// Ensures that at least `len` bytes of disk space are allocated for the
  /// file. After a successful call to `allocate`, subsequent writes to the
  /// file within the specified length are guaranteed not to fail because of
  /// lack of disk space.
  ///
  /// On most platforms the file's logical size is also extended to `len`
  /// bytes. On Windows, if the file's existing cluster-aligned allocation
  /// already covers `len`, the logical size is left unchanged to work around
  /// buffered-I/O quirks observed when the end-of-file pointer is moved
  /// inside an already-allocated cluster.
  fn allocate(&self, len: u64) -> Result<()>;

  /// Acquires a shared lock on the file, blocking until the lock can be
  /// acquired.
  fn lock_shared(&self) -> Result<()>;

  /// Acquires an exclusive lock on the file, blocking until the lock can be
  /// acquired.
  ///
  /// This is the blocking counterpart of [`FileExt::try_lock`]. It mirrors
  /// [`std::fs::File::lock`].
  fn lock(&self) -> Result<()>;

  /// Attempts to acquire a shared lock on the file, without blocking.
  ///
  /// Returns `Ok(())` if the lock was acquired, or
  /// `Err(`[`TryLockError::WouldBlock`](crate::TryLockError::WouldBlock)`)`
  /// if the file is currently locked. Mirrors
  /// [`std::fs::File::try_lock_shared`].
  fn try_lock_shared(&self) -> std::result::Result<(), TryLockError>;

  /// Attempts to acquire an exclusive lock on the file, without blocking.
  ///
  /// Returns `Ok(())` if the lock was acquired, or
  /// `Err(`[`TryLockError::WouldBlock`](crate::TryLockError::WouldBlock)`)`
  /// if the file is currently locked. Mirrors [`std::fs::File::try_lock`].
  fn try_lock(&self) -> std::result::Result<(), TryLockError>;

  /// Releases any lock held on the file. The lock is also released
  /// automatically when the file handle is closed.
  fn unlock(&self) -> Result<()>;
}

impl<F: FileExt + ?Sized> FileExt for &F {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn allocated_size(&self) -> Result<u64> {
    <F as FileExt>::allocated_size(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn allocate(&self, len: u64) -> Result<()> {
    <F as FileExt>::allocate(*self, len)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lock_shared(&self) -> Result<()> {
    <F as FileExt>::lock_shared(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lock(&self) -> Result<()> {
    <F as FileExt>::lock(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_lock_shared(&self) -> std::result::Result<(), TryLockError> {
    <F as FileExt>::try_lock_shared(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_lock(&self) -> std::result::Result<(), TryLockError> {
    <F as FileExt>::try_lock(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn unlock(&self) -> Result<()> {
    <F as FileExt>::unlock(*self)
  }
}

/// Extension trait for file which provides allocation and locking methods.
///
/// ## Notes on File Locks
///
/// This library provides whole-file locks in both shared (read) and exclusive
/// (read-write) varieties.
///
/// File locks are a cross-platform hazard since the file lock APIs exposed by
/// operating system kernels vary in subtle and not-so-subtle ways.
///
/// The API exposed by this library can be safely used across platforms as long
/// as the following rules are followed:
///
///   * Multiple locks should not be created on an individual `File` instance
///     concurrently.
///   * Duplicated files should not be locked without great care.
///   * Files to be locked should be opened with at least read or write
///     permissions.
///   * File locks may only be relied upon to be advisory.
///
/// File locks are released automatically when the file handle is closed (for
/// example when the owning `File` is dropped), so calling [`AsyncFileExt::unlock`]
/// explicitly is optional.
///
/// File locks are implemented with
/// [`flock(2)`](http://man7.org/linux/man-pages/man2/flock.2.html) on Unix and
/// [`LockFileEx`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-lockfileex)
/// on Windows. The `lock_*` and `try_lock_*` methods are synchronous because
/// the underlying system calls are blocking. The separate
/// [`AsyncFileExt::unlock_async`] method is provided for convenience inside
/// async code, but the underlying `unlock` syscall is still blocking.
///
/// This trait is sealed and cannot be implemented for types outside of `fs4`.
pub trait AsyncFileExt: sealed::Sealed {
  /// Returns the amount of physical space allocated for a file.
  fn allocated_size(&self) -> impl core::future::Future<Output = Result<u64>>;

  /// Ensures that at least `len` bytes of disk space are allocated for the
  /// file. After a successful call to `allocate`, subsequent writes to the
  /// file within the specified length are guaranteed not to fail because of
  /// lack of disk space.
  ///
  /// On most platforms the file's logical size is also extended to `len`
  /// bytes. On Windows, if the file's existing cluster-aligned allocation
  /// already covers `len`, the logical size is left unchanged to work around
  /// buffered-I/O quirks observed when the end-of-file pointer is moved
  /// inside an already-allocated cluster.
  fn allocate(&self, len: u64) -> impl core::future::Future<Output = Result<()>>;

  /// Acquires a shared lock on the file, blocking until the lock can be
  /// acquired.
  fn lock_shared(&self) -> Result<()>;

  /// Acquires an exclusive lock on the file, blocking until the lock can be
  /// acquired. Mirrors [`std::fs::File::lock`].
  fn lock(&self) -> Result<()>;

  /// Attempts to acquire a shared lock on the file, without blocking.
  ///
  /// Returns `Ok(())` if the lock was acquired, or
  /// `Err(`[`TryLockError::WouldBlock`](crate::TryLockError::WouldBlock)`)`
  /// if the file is currently locked.
  fn try_lock_shared(&self) -> std::result::Result<(), crate::TryLockError>;

  /// Attempts to acquire an exclusive lock on the file, without blocking.
  ///
  /// Returns `Ok(())` if the lock was acquired, or
  /// `Err(`[`TryLockError::WouldBlock`](crate::TryLockError::WouldBlock)`)`
  /// if the file is currently locked.
  fn try_lock(&self) -> std::result::Result<(), crate::TryLockError>;

  /// Releases any lock held on the file. The lock is also released
  /// automatically when the file handle is closed.
  fn unlock(&self) -> Result<()>;

  /// Releases any lock held on the file.
  ///
  /// **Note:** This method is not truly async; the underlying system call is
  /// still blocking. It exists for convenience when used from an async
  /// context.
  fn unlock_async(&self) -> impl core::future::Future<Output = Result<()>>;
}

impl<F: AsyncFileExt + ?Sized> AsyncFileExt for &F {
  #[cfg_attr(not(tarpaulin), inline(always))]
  async fn allocated_size(&self) -> Result<u64> {
    <F as AsyncFileExt>::allocated_size(*self).await
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  async fn allocate(&self, len: u64) -> Result<()> {
    <F as AsyncFileExt>::allocate(*self, len).await
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lock_shared(&self) -> Result<()> {
    <F as AsyncFileExt>::lock_shared(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lock(&self) -> Result<()> {
    <F as AsyncFileExt>::lock(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_lock_shared(&self) -> std::result::Result<(), crate::TryLockError> {
    <F as AsyncFileExt>::try_lock_shared(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_lock(&self) -> std::result::Result<(), crate::TryLockError> {
    <F as AsyncFileExt>::try_lock(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn unlock(&self) -> Result<()> {
    <F as AsyncFileExt>::unlock(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  async fn unlock_async(&self) -> Result<()> {
    <F as AsyncFileExt>::unlock_async(*self).await
  }
}

/// A heap-allocated, dynamically-typed `Send` future used by
/// [`DynAsyncFileExt`] to keep its methods object-safe.
pub type BoxFuture<'a, T> = core::pin::Pin<Box<dyn core::future::Future<Output = T> + Send + 'a>>;

/// Object-safe variant of [`AsyncFileExt`] returning boxed `Send` futures, so
/// it can be used behind a trait object (e.g. `Box<dyn DynAsyncFileExt>` or
/// `&dyn DynAsyncFileExt`).
///
/// [`AsyncFileExt`] uses return-position `impl Future`, which is not
/// object-safe; this trait wraps the same operations behind
/// [`BoxFuture`]s so the trait *can* be used as a trait object. Every type
/// that implements [`AsyncFileExt`] also implements `DynAsyncFileExt`.
///
/// Prefer [`AsyncFileExt`] for generic code (no allocation, no dynamic
/// dispatch); reach for `DynAsyncFileExt` only when type erasure is
/// required.
///
/// This trait is sealed and cannot be implemented for types outside of `fs4`.
pub trait DynAsyncFileExt: sealed::Sealed {
  /// Returns the amount of physical space allocated for a file.
  fn allocated_size(&self) -> BoxFuture<'_, Result<u64>>;

  /// Ensures that at least `len` bytes of disk space are allocated for the
  /// file. After a successful call to `allocate`, subsequent writes to the
  /// file within the specified length are guaranteed not to fail because of
  /// lack of disk space.
  ///
  /// On most platforms the file's logical size is also extended to `len`
  /// bytes. On Windows, if the file's existing cluster-aligned allocation
  /// already covers `len`, the logical size is left unchanged to work around
  /// buffered-I/O quirks observed when the end-of-file pointer is moved
  /// inside an already-allocated cluster.
  fn allocate(&self, len: u64) -> BoxFuture<'_, Result<()>>;

  /// Acquires a shared lock on the file, blocking until the lock can be
  /// acquired.
  fn lock_shared(&self) -> Result<()>;

  /// Acquires an exclusive lock on the file, blocking until the lock can be
  /// acquired. Mirrors [`std::fs::File::lock`].
  fn lock(&self) -> Result<()>;

  /// Attempts to acquire a shared lock on the file, without blocking.
  ///
  /// Returns `Ok(())` if the lock was acquired, or
  /// `Err(`[`TryLockError::WouldBlock`](crate::TryLockError::WouldBlock)`)`
  /// if the file is currently locked.
  fn try_lock_shared(&self) -> std::result::Result<(), crate::TryLockError>;

  /// Attempts to acquire an exclusive lock on the file, without blocking.
  ///
  /// Returns `Ok(())` if the lock was acquired, or
  /// `Err(`[`TryLockError::WouldBlock`](crate::TryLockError::WouldBlock)`)`
  /// if the file is currently locked.
  fn try_lock(&self) -> std::result::Result<(), crate::TryLockError>;

  /// Releases any lock held on the file. The lock is also released
  /// automatically when the file handle is closed.
  fn unlock(&self) -> Result<()>;

  /// Releases any lock held on the file.
  ///
  /// **Note:** This method is not truly async; the underlying system call is
  /// still blocking. It exists for convenience when used from an async
  /// context.
  fn unlock_async(&self) -> BoxFuture<'_, Result<()>>;
}

impl<F: DynAsyncFileExt + ?Sized> DynAsyncFileExt for &F {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn allocated_size(&self) -> BoxFuture<'_, Result<u64>> {
    <F as DynAsyncFileExt>::allocated_size(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn allocate(&self, len: u64) -> BoxFuture<'_, Result<()>> {
    <F as DynAsyncFileExt>::allocate(*self, len)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lock_shared(&self) -> Result<()> {
    <F as DynAsyncFileExt>::lock_shared(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lock(&self) -> Result<()> {
    <F as DynAsyncFileExt>::lock(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_lock_shared(&self) -> std::result::Result<(), crate::TryLockError> {
    <F as DynAsyncFileExt>::try_lock_shared(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_lock(&self) -> std::result::Result<(), crate::TryLockError> {
    <F as DynAsyncFileExt>::try_lock(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn unlock(&self) -> Result<()> {
    <F as DynAsyncFileExt>::unlock(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn unlock_async(&self) -> BoxFuture<'_, Result<()>> {
    <F as DynAsyncFileExt>::unlock_async(*self)
  }
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
