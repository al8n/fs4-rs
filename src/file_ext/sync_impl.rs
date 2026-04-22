macro_rules! file_ext {
    ($file:ty, $file_name:literal) => {
        use std::io::Result;

        #[doc = concat!("Extension trait for `", $file_name, "` which provides allocation and locking methods.")]
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
        pub trait FileExt {
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
            fn try_lock_shared(&self) -> std::result::Result<(), crate::TryLockError>;

            /// Attempts to acquire an exclusive lock on the file, without blocking.
            ///
            /// Returns `Ok(())` if the lock was acquired, or
            /// `Err(`[`TryLockError::WouldBlock`](crate::TryLockError::WouldBlock)`)`
            /// if the file is currently locked. Mirrors [`std::fs::File::try_lock`].
            fn try_lock(&self) -> std::result::Result<(), crate::TryLockError>;

            /// Releases any lock held on the file. The lock is also released
            /// automatically when the file handle is closed.
            fn unlock(&self) -> Result<()>;
        }

        impl FileExt for $file {
            fn allocated_size(&self) -> Result<u64> {
                sys::allocated_size(self)
            }
            fn allocate(&self, len: u64) -> Result<()> {
                sys::allocate(self, len)
            }
            fn lock_shared(&self) -> Result<()> {
                sys::lock_shared(self)
            }
            fn lock(&self) -> Result<()> {
                sys::lock(self)
            }
            fn try_lock_shared(&self) -> std::result::Result<(), crate::TryLockError> {
                sys::try_lock_shared(self)
            }
            fn try_lock(&self) -> std::result::Result<(), crate::TryLockError> {
                sys::try_lock(self)
            }
            fn unlock(&self) -> Result<()> {
                sys::unlock(self)
            }
        }
    }
}

macro_rules! test_mod {
    ($($use_stmt:item)*) => {
        #[cfg(test)]
        mod test {
            extern crate tempfile;

            use super::*;
            use crate::{allocation_granularity, statvfs, FsStats, TryLockError};

            $(
                $use_stmt
            )*

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
            #[test]
            fn filesystem_space() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let FsStats {
                    free_space,
                    available_space,
                    total_space,
                    ..
                } = statvfs(tempdir.path()).unwrap();

                assert!(total_space > free_space);
                assert!(total_space > available_space);
                assert!(available_space <= free_space);
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
