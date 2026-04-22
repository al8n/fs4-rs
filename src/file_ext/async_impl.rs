macro_rules! async_file_ext {
    ($file: ty, $file_name: literal) => {
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
        pub trait AsyncFileExt {
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

        impl AsyncFileExt for $file {
            async fn allocated_size(&self) -> Result<u64> {
                sys::allocated_size(self).await
            }
            async fn allocate(&self, len: u64) -> Result<()> {
                sys::allocate(self, len).await
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

            async fn unlock_async(&self) -> Result<()> {
                sys::unlock(self)
            }
        }
    }
}

macro_rules! test_mod {
     ($annotation:meta, $($use_stmt:item)*) => {
        #[cfg(test)]
        mod test {
            extern crate tempfile;
            use crate::{allocation_granularity, TryLockError};

            $(
                $use_stmt
            )*

            /// Tests shared file lock operations.
            #[$annotation]
            async fn lock_shared() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file1 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();
                let file2 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();
                let file3 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();

                // Concurrent shared access is OK, but not shared and exclusive.
                file1.lock_shared().unwrap();
                file2.lock_shared().unwrap();
                assert!(matches!(file3.try_lock(), Err(TryLockError::WouldBlock)));
                file1.unlock().unwrap();
                assert!(matches!(file3.try_lock(), Err(TryLockError::WouldBlock)));

                // Once all shared file locks are dropped, an exclusive lock may be created;
                file2.unlock().unwrap();
                file3.lock().unwrap();
            }

            /// Tests exclusive file lock operations.
            #[$annotation]
            async fn lock_exclusive() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file1 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();
                let file2 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();

                // No other access is possible once an exclusive lock is created.
                file1.lock().unwrap();
                assert!(matches!(file2.try_lock(), Err(TryLockError::WouldBlock)));
                assert!(matches!(
                    file2.try_lock_shared(),
                    Err(TryLockError::WouldBlock),
                ));

                // Once the exclusive lock is dropped, the second file is able to create a lock.
                file1.unlock().unwrap();
                file2.lock().unwrap();
            }

            /// Tests that a lock is released after the file that owns it is dropped.
            #[$annotation]
            async fn lock_cleanup() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file1 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();
                let file2 = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();

                file1.lock().unwrap();
                assert!(matches!(
                    file2.try_lock_shared(),
                    Err(TryLockError::WouldBlock),
                ));

                // Drop file1; the lock should be released.
                drop(file1);
                file2.lock_shared().unwrap();
            }

            /// Tests file allocation.
            #[$annotation]
            async fn allocate() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();
                let blksize = allocation_granularity(&path).unwrap();

                // New files are created with no allocated size.
                assert_eq!(0, file.allocated_size().await.unwrap());
                assert_eq!(0, file.metadata().await.unwrap().len());

                // Allocate space for the file, checking that the allocated size steps
                // up by block size, and the file length matches the allocated size.

                file.allocate(2 * blksize - 1).await.unwrap();
                assert_eq!(2 * blksize, file.allocated_size().await.unwrap());
                assert_eq!(2 * blksize - 1, file.metadata().await.unwrap().len());

                // Truncate the file, checking that the allocated size steps down by
                // block size.

                file.set_len(blksize + 1).await.unwrap();
                assert_eq!(2 * blksize, file.allocated_size().await.unwrap());
                assert_eq!(blksize + 1, file.metadata().await.unwrap().len());
            }

            /// Regression for issue #13: on Windows, re-`allocate`-ing
            /// inside an already-allocated cluster must not move the EOF
            /// pointer (the old path called `set_len`, which triggered
            /// Windows buffered-I/O quirks). The trait doc carves this
            /// out from the general "file size is at least `len`"
            /// contract, so this test asserts the strict invariant on
            /// Windows and a loose one on Unix.
            #[$annotation]
            async fn allocate_preserves_eof_within_cluster() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();
                let blksize = allocation_granularity(&path).unwrap();

                file.allocate(2 * blksize - 1).await.unwrap();
                assert_eq!(2 * blksize - 1, file.metadata().await.unwrap().len());

                file.allocate(2 * blksize).await.unwrap();

                #[cfg(windows)]
                assert_eq!(
                    2 * blksize - 1,
                    file.metadata().await.unwrap().len(),
                    "Windows allocate must not extend EOF inside an already-allocated cluster (#13)",
                );
                #[cfg(unix)]
                assert!(file.metadata().await.unwrap().len() >= 2 * blksize - 1);
            }

            /// Regression test for issue #15: re-allocating the same length must
            /// not fail on macOS.
            #[$annotation]
            async fn allocate_idempotent() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();
                let blksize = allocation_granularity(&path).unwrap();

                file.allocate(2 * blksize).await.unwrap();
                file.allocate(2 * blksize).await.unwrap();
                file.allocate(blksize).await.unwrap();
                assert!(file.metadata().await.unwrap().len() >= 2 * blksize);
            }

            /// Checks filesystem space methods.
            ///
            /// Uses a single `statvfs` call + destructure so the three
            /// numbers are read atomically; calling `free_space` /
            /// `available_space` / `total_space` separately makes the
            /// assertions race with concurrent filesystem activity
            /// from other tests.
            ///
            /// Does not assert `available_space <= free_space`: on
            /// macOS APFS the kernel reports `f_bavail > f_bfree`
            /// because purgeable space (snapshots, cached data) is
            /// counted as available but not as free, so the usual
            /// POSIX invariant does not hold.
            #[$annotation]
            async fn filesystem_space() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let crate::FsStats {
                    free_space,
                    available_space,
                    total_space,
                    ..
                } = crate::statvfs(tempdir.path()).unwrap();

                assert!(total_space >= free_space);
                assert!(total_space >= available_space);
            }
        }
    };
}

cfg_async_std! {
    pub(crate) mod async_std_impl;
}

cfg_fs_err2_tokio! {
    pub(crate) mod fs_err2_tokio_impl;
}

cfg_fs_err3_tokio! {
    pub(crate) mod fs_err3_tokio_impl;
}

cfg_smol! {
    pub(crate) mod smol_impl;
}

cfg_tokio! {
    pub(crate) mod tokio_impl;
}
