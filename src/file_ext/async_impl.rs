macro_rules! async_file_ext {
    ($file: ty, $file_name: literal) => {
        use std::io::Result;

        impl $crate::sealed::Sealed for $file {}

        impl $crate::AsyncFileExt for $file {
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

            fn try_lock_shared(&self) -> std::result::Result<(), $crate::TryLockError> {
                sys::try_lock_shared(self)
            }

            fn try_lock(&self) -> std::result::Result<(), $crate::TryLockError> {
                sys::try_lock(self)
            }

            fn unlock(&self) -> Result<()> {
                sys::unlock(self)
            }

            async fn unlock_async(&self) -> Result<()> {
                sys::unlock(self)
            }
        }

        impl $crate::DynAsyncFileExt for $file {
            #[cfg_attr(not(tarpaulin), inline(always))]
            fn allocated_size(&self) -> $crate::BoxFuture<'_, Result<u64>> {
                Box::pin(<Self as $crate::AsyncFileExt>::allocated_size(self))
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn allocate(&self, len: u64) -> $crate::BoxFuture<'_, Result<()>> {
                Box::pin(<Self as $crate::AsyncFileExt>::allocate(self, len))
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn lock_shared(&self) -> Result<()> {
                <Self as $crate::AsyncFileExt>::lock_shared(self)
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn lock(&self) -> Result<()> {
                <Self as $crate::AsyncFileExt>::lock(self)
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn try_lock_shared(&self) -> std::result::Result<(), $crate::TryLockError> {
                <Self as $crate::AsyncFileExt>::try_lock_shared(self)
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn try_lock(&self) -> std::result::Result<(), $crate::TryLockError> {
                <Self as $crate::AsyncFileExt>::try_lock(self)
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn unlock(&self) -> Result<()> {
                <Self as $crate::AsyncFileExt>::unlock(self)
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn unlock_async(&self) -> $crate::BoxFuture<'_, Result<()>> {
                Box::pin(<Self as $crate::AsyncFileExt>::unlock_async(self))
            }
        }
    }
}

macro_rules! test_mod {
     ($annotation:meta, $($use_stmt:item)*) => {
        #[cfg(test)]
        mod test {
            extern crate tempfile;

            use crate::{allocation_granularity, TryLockError, AsyncFileExt};

            $(
                $use_stmt
            )*

            /// Exercises `impl<F: AsyncFileExt + ?Sized> AsyncFileExt for &F`.
            /// The other tests call methods directly on the concrete file type
            /// and resolve to `<File as AsyncFileExt>::*`, so without this the
            /// blanket-impl bodies are unreached.
            #[$annotation]
            async fn async_file_ext_blanket_for_ref() {
                async fn drive<F: AsyncFileExt + ?Sized>(f: &F, len: u64) {
                    f.lock_shared().unwrap();
                    f.unlock().unwrap();
                    f.lock().unwrap();
                    f.unlock().unwrap();
                    f.try_lock_shared().unwrap();
                    f.unlock().unwrap();
                    f.try_lock().unwrap();
                    f.unlock_async().await.unwrap();
                    f.allocate(len).await.unwrap();
                    let _ = f.allocated_size().await.unwrap();
                }

                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();
                let blksize = allocation_granularity(&path).unwrap();

                // `&&file` binds `F = &File`, so calls go through the blanket.
                drive(&&file, blksize).await;
            }

            /// Drives every `DynAsyncFileExt` method twice: once on the
            /// concrete file type to cover the per-backend macro impl, and
            /// once through `&File` to cover the `for &F` blanket.
            #[$annotation]
            async fn dyn_async_file_ext_smoke() {
                use crate::DynAsyncFileExt;

                async fn drive<F: DynAsyncFileExt + ?Sized>(f: &F, len: u64) {
                    f.lock_shared().unwrap();
                    f.unlock().unwrap();
                    f.lock().unwrap();
                    f.unlock().unwrap();
                    f.try_lock_shared().unwrap();
                    f.unlock().unwrap();
                    f.try_lock().unwrap();
                    f.unlock_async().await.unwrap();
                    f.allocate(len).await.unwrap();
                    let _ = f.allocated_size().await.unwrap();
                }

                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();
                let blksize = allocation_granularity(&path).unwrap();

                // `&file` binds `F = File`, exercising the per-backend impl.
                drive(&file, blksize).await;
                // `&&file` binds `F = &File`, exercising the blanket impl.
                drive(&&file, blksize).await;
            }

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

            /// `unlock_async` is a thin async wrapper over the
            /// blocking `unlock` syscall. Verifies the forwarder
            /// actually releases the lock.
            #[$annotation]
            async fn unlock_async_releases_lock() {
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
                assert!(matches!(file2.try_lock(), Err(TryLockError::WouldBlock)));

                file1.unlock_async().await.unwrap();

                file2.lock().unwrap();
                file2.unlock_async().await.unwrap();
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
            #[$annotation]
            async fn allocate_reserves_blocks_on_sparse_file() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                let file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&path)
                    .await
                    .unwrap();
                let blksize = allocation_granularity(&path).unwrap();

                file.set_len(4 * blksize).await.unwrap();
                assert_eq!(4 * blksize, file.metadata().await.unwrap().len());
                assert_eq!(0, file.allocated_size().await.unwrap());

                file.allocate(4 * blksize).await.unwrap();

                assert!(
                    file.allocated_size().await.unwrap() >= 4 * blksize,
                    "allocate on a sparse file must reserve blocks",
                );
            }

            /// Exercises the `Err` arm of the Unix `fallocate` call
            /// by invoking `allocate` on a read-only file
            /// descriptor, which returns `EBADF`. Without this test
            /// the error conversion
            /// (`std::io::Error::from_raw_os_error`) and the
            /// `Err(...) => Err(...)` match arm are uncovered. Gated
            /// to Unix: Windows has a separate `allocate` with its
            /// own error path.
            #[cfg(unix)]
            #[$annotation]
            async fn allocate_forwards_fallocate_error() {
                let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
                let path = tempdir.path().join("fs4");
                // Create then close the file.
                drop(
                    fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&path)
                        .await
                        .unwrap(),
                );
                // Re-open read-only. `fallocate` requires a writable
                // fd; the syscall fails with EBADF and the error is
                // propagated through the match arm.
                let file = fs::OpenOptions::new()
                    .read(true)
                    .open(&path)
                    .await
                    .unwrap();
                let err = file.allocate(4096).await.unwrap_err();
                assert!(
                    err.raw_os_error().is_some(),
                    "expected a raw OS error from fallocate, got {err:?}",
                );
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
