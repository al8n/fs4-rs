macro_rules! allocate_size {
  ($file:ty) => {
    pub fn allocated_size(file: &$file) -> Result<u64> {
      unsafe {
        let mut info: FILE_STANDARD_INFO = mem::zeroed();

        let ret = GetFileInformationByHandleEx(
          file.as_raw_handle() as HANDLE,
          FileStandardInfo,
          &mut info as *mut _ as *mut _,
          mem::size_of::<FILE_STANDARD_INFO>() as u32,
        );

        if ret == 0 {
          Err(Error::last_os_error())
        } else {
          Ok(info.AllocationSize as u64)
        }
      }
    }
  };
}

macro_rules! allocate {
  ($file:ty) => {
    pub fn allocate(file: &$file, len: u64) -> Result<()> {
      // If the file already has `len` bytes of cluster-aligned
      // allocation, do nothing. Calling `set_len` in this branch
      // (to extend the EOF pointer to `len` when it lags behind
      // the `AllocationSize`) has been observed to trigger
      // Windows quirks on the subsequent buffered I/O path — see
      // issue #13. Skipping it is safe: the physical disk space
      // the caller asked for is already reserved, and
      // subsequent writes will extend EOF naturally.
      if allocated_size(file)? >= len {
        return Ok(());
      }
      unsafe {
        let mut info: FILE_ALLOCATION_INFO = mem::zeroed();
        info.AllocationSize = len as i64;
        let ret = SetFileInformationByHandle(
          file.as_raw_handle() as HANDLE,
          FileAllocationInfo,
          &mut info as *mut _ as *mut _,
          mem::size_of::<FILE_ALLOCATION_INFO>() as u32,
        );
        if ret == 0 {
          return Err(Error::last_os_error());
        }
      }
      if file.metadata()?.len() < len {
        file.set_len(len)
      } else {
        Ok(())
      }
    }
  };
}

macro_rules! test_mod {
    ($($use_stmt:item)*) => {
        #[cfg(test)]
        mod test {
          extern crate tempfile;

          use crate::TryLockError;

          $(
              $use_stmt
          )*

          /// A file handle may not be exclusively locked multiple times, or exclusively locked and then
          /// shared locked.
          #[test]
          fn lock_non_reentrant() {
              let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
              let path = tempdir.path().join("fs4");
              let file = fs::OpenOptions::new()
                  .read(true)
                  .write(true)
                  .create(true)
                  .open(path)
                  .unwrap();

              // Multiple exclusive locks fails.
              FileExt::lock(&file).unwrap();
              assert!(matches!(
                  FileExt::try_lock(&file),
                  Err(TryLockError::WouldBlock),
              ));
              FileExt::unlock(&file).unwrap();

              // Shared then Exclusive locks fails.
              FileExt::lock_shared(&file).unwrap();
              assert!(matches!(
                  FileExt::try_lock(&file),
                  Err(TryLockError::WouldBlock),
              ));
          }

          /// A file handle can hold an exclusive lock and any number of shared locks, all of which must
          /// be unlocked independently.
          #[test]
          fn lock_layering() {
              let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
              let path = tempdir.path().join("fs4");
              let file = fs::OpenOptions::new()
                  .read(true)
                  .write(true)
                  .create(true)
                  .open(path)
                  .unwrap();

              // Open two shared locks on the file, and then try and fail to open an exclusive lock.
              FileExt::lock(&file).unwrap();
              FileExt::lock_shared(&file).unwrap();
              FileExt::lock_shared(&file).unwrap();
              assert!(
                  matches!(FileExt::try_lock(&file), Err(TryLockError::WouldBlock)),
                  "the first try lock exclusive",
              );

              // Pop one of the shared locks and try again.
              FileExt::unlock(&file).unwrap();
              assert!(
                  matches!(FileExt::try_lock(&file), Err(TryLockError::WouldBlock)),
                  "pop the first shared lock",
              );

              // Pop the second shared lock and try again.
              FileExt::unlock(&file).unwrap();
              assert!(
                  matches!(FileExt::try_lock(&file), Err(TryLockError::WouldBlock)),
                  "pop the second shared lock",
              );

              // Pop the exclusive lock and finally succeed.
              FileExt::unlock(&file).unwrap();
              FileExt::lock(&file).unwrap();
          }

          /// A file handle with multiple open locks will have all locks closed on drop.
          #[test]
          fn lock_layering_cleanup() {
              let tempdir = tempfile::TempDir::with_prefix("fs4").unwrap();
              let path = tempdir.path().join("fs4");
              let file1 = fs::OpenOptions::new()
                  .read(true)
                  .write(true)
                  .create(true)
                  .open(&path)
                  .unwrap();
              let file2 = fs::OpenOptions::new()
                  .read(true)
                  .write(true)
                  .create(true)
                  .open(&path)
                  .unwrap();

              // Open two shared locks on the file, and then try and fail to open an exclusive lock.
              FileExt::lock_shared(&file1).unwrap();
              assert!(matches!(
                  FileExt::try_lock(&file2),
                  Err(TryLockError::WouldBlock),
              ));

              drop(file1);
              FileExt::lock(&file2).unwrap();
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
