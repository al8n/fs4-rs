// Expanded inside the `sync_impl` and `async_impl` submodules, both of
// which are feature-gated. With `--no-default-features` (nothing but
// the filesystem-stats API enabled), this macro has no caller, so
// silence the resulting unused-macro lint.
//
// The helpers take `&$file` and only cross the raw-handle boundary
// (`as_raw_handle() as HANDLE`) at the FFI call site. Routing through
// `AsHandle` / `BorrowedHandle` would be ideal but isn't uniformly
// available: `async_std::fs::File`, `smol::fs::File`, and the
// `fs_err{2,3}::tokio::File` wrappers only implement `AsRawHandle`
// upstream, so we keep a single code path.
#[allow(unused_macros)]
macro_rules! lock_impl {
  ($file: ty) => {
    pub fn lock_shared(file: &$file) -> std::io::Result<()> {
      lock_file(file, 0)
    }

    pub fn lock(file: &$file) -> std::io::Result<()> {
      lock_file(file, LOCKFILE_EXCLUSIVE_LOCK)
    }

    pub fn try_lock_shared(file: &$file) -> std::result::Result<(), $crate::TryLockError> {
      try_lock_file(file, LOCKFILE_FAIL_IMMEDIATELY)
    }

    pub fn try_lock(file: &$file) -> std::result::Result<(), $crate::TryLockError> {
      try_lock_file(file, LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY)
    }

    pub fn unlock(file: &$file) -> std::io::Result<()> {
      unsafe {
        let ret = UnlockFile(file.as_raw_handle() as HANDLE, 0, 0, !0, !0);
        if ret == 0 {
          Err(Error::last_os_error())
        } else {
          Ok(())
        }
      }
    }

    fn lock_file(file: &$file, flags: u32) -> std::io::Result<()> {
      unsafe {
        let mut overlapped = mem::zeroed();
        let ret = LockFileEx(
          file.as_raw_handle() as HANDLE,
          flags,
          0,
          !0,
          !0,
          &mut overlapped,
        );
        if ret == 0 {
          Err(Error::last_os_error())
        } else {
          Ok(())
        }
      }
    }

    fn try_lock_file(file: &$file, flags: u32) -> std::result::Result<(), $crate::TryLockError> {
      unsafe {
        let mut overlapped = mem::zeroed();
        let ret = LockFileEx(
          file.as_raw_handle() as HANDLE,
          flags,
          0,
          !0,
          !0,
          &mut overlapped,
        );
        if ret == 0 {
          let err = Error::last_os_error();
          if err.raw_os_error()
            == Some(::windows_sys::Win32::Foundation::ERROR_LOCK_VIOLATION as i32)
          {
            return Err($crate::TryLockError::WouldBlock);
          }

          Err($crate::TryLockError::Error(err))
        } else {
          Ok(())
        }
      }
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
use std::io::{Error, Result};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::Win32::Storage::FileSystem::{
  GetDiskFreeSpaceExW, GetDiskFreeSpaceW, GetVolumePathNameW,
};

fn volume_path(path: &Path, volume_path: &mut [u16]) -> Result<()> {
  let path_utf8: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
  unsafe {
    let ret = GetVolumePathNameW(
      path_utf8.as_ptr(),
      volume_path.as_mut_ptr(),
      volume_path.len() as u32,
    );
    if ret == 0 {
      Err(Error::last_os_error())
    } else {
      Ok(())
    }
  }
}

pub fn statvfs(path: &Path) -> Result<FsStats> {
  let root_path: &mut [u16] = &mut [0; 261];
  volume_path(path, root_path)?;
  unsafe {
    // `GetDiskFreeSpaceExW` returns three distinct numbers we must not
    // conflate on quota-enabled volumes: caller-scoped free (honours
    // per-user quotas) → `available_space`; caller-scoped total
    // (unused here, see `total_space` below); and volume-wide free
    // (quota-independent) → `free_space`.
    let mut bytes_available_to_caller = 0u64;
    let mut _caller_total_bytes = 0u64;
    let mut free_bytes_on_volume = 0u64;
    let ret = GetDiskFreeSpaceExW(
      root_path.as_ptr(),
      &mut bytes_available_to_caller,
      &mut _caller_total_bytes,
      &mut free_bytes_on_volume,
    );
    if ret == 0 {
      return Err(Error::last_os_error());
    }

    let mut sectors_per_cluster = 0;
    let mut bytes_per_sector = 0;
    let mut _number_of_free_clusters = 0;
    let mut total_number_of_clusters = 0;
    let ret = GetDiskFreeSpaceW(
      root_path.as_ptr(),
      &mut sectors_per_cluster,
      &mut bytes_per_sector,
      &mut _number_of_free_clusters,
      &mut total_number_of_clusters,
    );
    if ret == 0 {
      Err(Error::last_os_error())
    } else {
      let bytes_per_cluster = sectors_per_cluster as u64 * bytes_per_sector as u64;
      let total_space = bytes_per_cluster.saturating_mul(total_number_of_clusters as u64);
      Ok(FsStats {
        free_space: free_bytes_on_volume,
        available_space: bytes_available_to_caller,
        total_space,
        allocation_granularity: bytes_per_cluster,
      })
    }
  }
}
