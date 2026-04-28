macro_rules! file_ext {
  ($file:ty, $file_name:literal) => {
    use std::io::Result;

    impl $crate::sealed::Sealed for $file {}

    impl $crate::FileExt for $file {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn allocated_size(&self) -> Result<u64> {
        sys::allocated_size(self)
      }
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn allocate(&self, len: u64) -> Result<()> {
        sys::allocate(self, len)
      }
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn lock_shared(&self) -> Result<()> {
        sys::lock_shared(self)
      }
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn lock(&self) -> Result<()> {
        sys::lock(self)
      }
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn try_lock_shared(&self) -> std::result::Result<(), $crate::TryLockError> {
        sys::try_lock_shared(self)
      }
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn try_lock(&self) -> std::result::Result<(), $crate::TryLockError> {
        sys::try_lock(self)
      }
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn unlock(&self) -> Result<()> {
        sys::unlock(self)
      }
    }
  };
}

macro_rules! test_mod {
    ($($use_stmt:item)*) => {
        #[cfg(test)]
        mod test {
            extern crate tempfile;

            use crate::{allocation_granularity, statvfs, FsStats, TryLockError, FileExt};

            $(
                $use_stmt
            )*

            /// Exercises `impl<F: FileExt + ?Sized> FileExt for &F`. Without this
            /// every method body in the blanket impl is uncovered, since the
            /// other tests call methods directly on the concrete file type and
            /// resolve to `<File as FileExt>::*`.
            #[test]
            fn file_ext_blanket_for_ref() {
                fn drive<F: FileExt + ?Sized>(f: &F, len: u64) {
                    f.lock_shared().unwrap();
                    f.unlock().unwrap();
                    f.lock().unwrap();
                    f.unlock().unwrap();
                    f.try_lock_shared().unwrap();
                    f.unlock().unwrap();
                    f.try_lock().unwrap();
                    f.unlock().unwrap();
                    f.allocate(len).unwrap();
                    let _ = f.allocated_size().unwrap();
                }

                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();
                let blksize = allocation_granularity(&path).unwrap();

                // `&&file` makes the inner `F` bind to `&File`, so all method
                // calls resolve to the `<&File as FileExt>` blanket impl.
                drive(&&file, blksize);
            }

            /// Tests shared file lock operations.
            #[test]
            fn lock_shared() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file1 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();
                let file2 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();
                let file3 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();

                // Concurrent shared access is OK, but not shared and exclusive.
                FileExt::lock_shared(&file1).unwrap();
                FileExt::lock_shared(&file2).unwrap();
                assert!(matches!(
                    FileExt::try_lock(&file3),
                    Err(TryLockError::WouldBlock),
                ));
                FileExt::unlock(&file1).unwrap();
                assert!(matches!(
                    FileExt::try_lock(&file3),
                    Err(TryLockError::WouldBlock),
                ));

                // Once all shared file locks are dropped, an exclusive lock may be created;
                FileExt::unlock(&file2).unwrap();
                FileExt::lock(&file3).unwrap();
            }

            /// Tests exclusive file lock operations.
            #[test]
            fn lock_exclusive() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file1 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();
                let file2 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();

                // No other access is possible once an exclusive lock is created.
                FileExt::lock(&file1).unwrap();
                assert!(matches!(
                    FileExt::try_lock(&file2),
                    Err(TryLockError::WouldBlock),
                ));
                assert!(matches!(
                    FileExt::try_lock_shared(&file2),
                    Err(TryLockError::WouldBlock),
                ));

                // Once the exclusive lock is dropped, the second file is able to create a lock.
                FileExt::unlock(&file1).unwrap();
                FileExt::lock(&file2).unwrap();
            }

            /// Tests that a lock is released after the file that owns it is dropped.
            #[test]
            fn lock_cleanup() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file1 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();
                let file2 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();

                FileExt::lock(&file1).unwrap();
                assert!(matches!(
                    FileExt::try_lock_shared(&file2),
                    Err(TryLockError::WouldBlock),
                ));

                // Drop file1; the lock should be released.
                drop(file1);
                FileExt::lock_shared(&file2).unwrap();
            }

            /// Tests file allocation.
            #[test]
            fn allocate() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();
                let blksize = allocation_granularity(&path).unwrap();

                // New files are created with no allocated size.
                assert_eq!(0, FileExt::allocated_size(&file).unwrap());
                assert_eq!(0, file.metadata().unwrap().len());

                // Allocate space for the file, checking that the allocated size steps
                // up by block size, and the file length matches the allocated size.

                FileExt::allocate(&file, 2 * blksize - 1).unwrap();
                assert_eq!(2 * blksize, FileExt::allocated_size(&file).unwrap());
                assert_eq!(2 * blksize - 1, file.metadata().unwrap().len());

                // Truncate the file, checking that the allocated size steps down by
                // block size.

                file.set_len(blksize + 1).unwrap();
                assert_eq!(2 * blksize, FileExt::allocated_size(&file).unwrap());
                assert_eq!(blksize + 1, file.metadata().unwrap().len());
            }

            /// Regression for issue #13: on Windows, re-`allocate`-ing inside
            /// an already-allocated cluster must not move the EOF pointer
            /// (the old behaviour called `set_len`, which triggered Windows
            /// buffered-I/O quirks). The trait doc explicitly carves this
            /// behaviour out from the general "file size is at least `len`"
            /// contract, so this test asserts the strict invariant on
            /// Windows and a loose one on Unix.
            #[test]
            fn allocate_preserves_eof_within_cluster() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();
                let blksize = allocation_granularity(&path).unwrap();

                // Reserve cluster-aligned space but leave EOF below the
                // cluster boundary — the precondition reported in #13.
                FileExt::allocate(&file, 2 * blksize - 1).unwrap();
                assert_eq!(2 * blksize - 1, file.metadata().unwrap().len());

                // Request `allocate` up to the cluster boundary. Before
                // the fix, Windows would `set_len(2*blksize)`; after, it
                // short-circuits since `allocated_size >= len`.
                FileExt::allocate(&file, 2 * blksize).unwrap();

                #[cfg(windows)]
                assert_eq!(
                    2 * blksize - 1,
                    file.metadata().unwrap().len(),
                    "Windows allocate must not extend EOF inside an already-allocated cluster (#13)",
                );
                #[cfg(unix)]
                assert!(file.metadata().unwrap().len() >= 2 * blksize - 1);
            }

            /// Regression: `allocate` on a sparse file must reserve
            /// blocks even when logical EOF already covers `len`. The
            /// previous Unix implementation short-circuited on
            /// `metadata().len()`, which for a file extended via
            /// `set_len` is true with zero blocks allocated, so the
            /// documented preallocation contract silently became a
            /// no-op. Gated to Linux where `set_len` reliably
            /// produces a sparse file and `st_blocks` reliably
            /// reflects `fallocate` reservations.
            #[cfg(target_os = "linux")]
            #[test]
            fn allocate_reserves_blocks_on_sparse_file() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();
                let blksize = allocation_granularity(&path).unwrap();

                // `set_len` extends logical EOF without reserving blocks:
                // the file is sparse with `len = 4 * blksize` and
                // `allocated_size = 0`.
                file.set_len(4 * blksize).unwrap();
                assert_eq!(4 * blksize, file.metadata().unwrap().len());
                assert_eq!(0, FileExt::allocated_size(&file).unwrap());

                FileExt::allocate(&file, 4 * blksize).unwrap();

                assert!(
                    FileExt::allocated_size(&file).unwrap() >= 4 * blksize,
                    "allocate on a sparse file must reserve blocks",
                );
            }

            /// Exercises the `Err` arm of the Unix `fallocate` call by
            /// invoking `allocate` on a read-only file descriptor,
            /// which returns `EBADF`. Without this test the error
            /// conversion (`std::io::Error::from_raw_os_error`) and
            /// the `Err(...) => Err(...)` match arm are uncovered.
            /// Gated to Unix: Windows has a separate `allocate` with
            /// its own error path.
            #[cfg(unix)]
            #[test]
            fn allocate_forwards_fallocate_error() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                // Create then close the file so it exists on disk.
                drop(
                    fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&path)
                        .unwrap(),
                );
                // Re-open read-only. `fallocate` requires the fd to
                // be writable, so the syscall fails with EBADF and
                // the error is propagated through the match arm.
                let file = fs::OpenOptions::new().read(true).open(&path).unwrap();
                let err = FileExt::allocate(&file, 4096).unwrap_err();
                assert!(
                    err.raw_os_error().is_some(),
                    "expected a raw OS error from fallocate, got {err:?}",
                );
            }

            /// Re-allocating the same length must not fail. Regression for issue #15.
            #[test]
            fn allocate_idempotent() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();
                let blksize = allocation_granularity(&path).unwrap();

                FileExt::allocate(&file, 2 * blksize).unwrap();
                FileExt::allocate(&file, 2 * blksize).unwrap();
                FileExt::allocate(&file, blksize).unwrap();
                assert!(file.metadata().unwrap().len() >= 2 * blksize);
            }

            /// Checks filesystem space methods.
            ///
            /// Does not assert `available_space <= free_space`: on
            /// macOS APFS the kernel reports `f_bavail > f_bfree`
            /// because purgeable space (snapshots, cached data) is
            /// counted as available but not as free, so the usual
            /// POSIX invariant does not hold.
            #[test]
            fn filesystem_space() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let FsStats {
                    free_space,
                    available_space,
                    total_space,
                    ..
                } = statvfs(tempdir.path()).unwrap();

                assert!(total_space >= free_space);
                assert!(total_space >= available_space);
            }

        }
    };
}

cfg_sync! {
    pub(crate) mod std_impl;
}

cfg_fs_err2! {
    pub(crate) mod fs_err2_impl;
}

cfg_fs_err3! {
    pub(crate) mod fs_err3_impl;
}
