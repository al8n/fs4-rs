# Releases

## 1.0.0

### Breakage

- Renamed `FileExt::lock_exclusive` / `AsyncFileExt::lock_exclusive` to
  `lock`, matching the stabilized [`std::fs::File::lock`] API.
- Renamed `FileExt::try_lock_exclusive` / `AsyncFileExt::try_lock_exclusive`
  to `try_lock`, matching [`std::fs::File::try_lock`].
- Changed the return type of `try_lock` and `try_lock_shared` from
  `std::io::Result<bool>` to `Result<(), TryLockError>`. `Ok(())` still
  indicates the lock was acquired; `Err(TryLockError::WouldBlock)` now
  indicates the lock is held by another handle. This matches the stable
  [`std::fs::File::try_lock`] signature (`Ok(false)` was the nightly
  shape prior to 1.89).
- Removed the top-level `lock_contended_error()` helper. Use
  `TryLockError::WouldBlock` instead.
- Flattened the `fs_std` module: the `FileExt` trait for
  `std::fs::File` now lives at the crate root. Update imports from
  `use fs4::fs_std::FileExt;` to `use fs4::FileExt;`. All other
  backends (`fs_err2`, `fs_err3`, `tokio`, `smol`, `async_std`,
  `fs_err2_tokio`, `fs_err3_tokio`) remain nested, since each
  defines its own `FileExt`/`AsyncFileExt` over a different concrete
  `File` type.

### Additions

- New `fs4::TryLockError` enum, mirroring [`std::fs::TryLockError`]
  (`Error(io::Error)` / `WouldBlock`). Includes `From<TryLockError>
  for io::Error` and `From<io::Error>` implementations.

### Fixes

- Fixed feature typos that made the crate fail to compile with only
  `fs-err2`, `fs-err3`, `fs-err2-tokio`, or `fs-err3-tokio` enabled
  (without `sync` / `tokio`).
- Added cygwin to the Unix `allocate` `target_os` set so builds stop
  failing with a missing `sys::allocate` on that target (#44).
- Added redox and cygwin to the async `allocate` fallback branch so
  it matches the sync variant.
- Moved AIX from the async `fallocate` branch to the `set_len`
  fallback branch, matching the sync implementation.
- Short-circuited `allocate` on Unix when the file is already at least
  `len` bytes long. Fixes macOS `fallocate` spuriously returning
  `ENOSPC` when re-calling `allocate(len)` on an existing file (#15).
- On Windows, skip the internal `set_len` call when the file's
  existing cluster-aligned allocation already covers `len`. Avoids
  the Windows buffered-I/O quirks described in #13; the trait doc now
  explicitly notes this behavior.
- Updated `html_root_url`.

### Dependency updates

- Bumped `windows-sys` from 0.59 to 0.61.

### Testing

- Removed all `#[bench]` functions from the test harness. They measured
  OS syscalls (`flock`, `LockFileEx`, `fallocate`, `statvfs`), not fs4
  code, and produced numbers that were dominated by the underlying
  filesystem. Dropping them lets the crate build and test on stable
  Rust (the bench harness was the only thing pinning nightly via
  `#![cfg_attr(test, feature(test))]`). `rust-toolchain.toml` is now
  `stable`.

## 0.13.0

### Breakage

- Make `try_lock_*` return `std::io::Result<bool>`, which is compatible with the upcoming `std::fs::File::try_lock*` in `std`.

[`std::fs::File::lock`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.lock
[`std::fs::File::try_lock`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock
[`std::fs::TryLockError`]: https://doc.rust-lang.org/std/fs/enum.TryLockError.html
