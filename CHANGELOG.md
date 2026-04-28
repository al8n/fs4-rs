# Releases

## 1.1.0

### Changes

- Consolidate `FileExt` and `AsyncFileExt` into single crate-root traits
  (`fs4::FileExt`, `fs4::AsyncFileExt`) instead of generating a distinct
  trait per backend module. The per-backend modules (`fs4::tokio`,
  `fs4::async_std`, `fs4::smol`, `fs4::fs_err2`, `fs4::fs_err3`,
  `fs4::fs_err2_tokio`, `fs4::fs_err3_tokio`) now re-export the unified
  crate-root trait. Method-call sites that import the trait via `use`
  continue to compile unchanged; code that named two backend traits as
  distinct types will see them unify.
- Add blanket impls `impl<F: FileExt + ?Sized> FileExt for &F` and
  `impl<F: AsyncFileExt + ?Sized> AsyncFileExt for &F`, so the
  extension methods are now callable through shared references.
- Seal `FileExt` and `AsyncFileExt` via a private `sealed::Sealed`
  supertrait, so the set of implementing types is closed to the
  concrete file types fs4 already supports (and references to them).
  This locks in the freedom to add methods to either trait in future
  minor releases without breaking downstream impls.
- Add `DynAsyncFileExt`, an object-safe mirror of `AsyncFileExt` whose
  async methods return `BoxFuture<'_, T>` (alias for
  `Pin<Box<dyn Future<Output = T> + Send + '_>>`). Use it whenever
  type erasure is needed (`Box<dyn DynAsyncFileExt>`,
  `&dyn DynAsyncFileExt`); prefer the static `AsyncFileExt` for
  generic code since it has no allocation or dynamic-dispatch
  overhead. Every type implementing `AsyncFileExt` also implements
  `DynAsyncFileExt`, and the trait is sealed.
- Mark the delegating methods `#[inline(always)]` (skipped under
  `tarpaulin` coverage builds).

## 1.0.1

### Fixes

- Unix `allocate`: short-circuit on allocated blocks
  (`metadata().blocks() * 512 >= len`) instead of logical EOF. The
  previous `metadata().len() >= len` check silently turned `allocate`
  into a no-op on sparse files (logical length large, zero blocks
  reserved), violating the documented preallocation guarantee. The
  new check still skips the macOS `F_PREALLOCATE` re-allocate-ENOSPC
  path from #15, since it asks the right question: "are the blocks
  already reserved?" Applies to both the sync and async
  implementations.
- Windows `statvfs`: route the three `GetDiskFreeSpaceExW` outputs
  correctly. `free_space` now comes from `lpTotalNumberOfFreeBytes`
  (volume-wide, quota-independent), `available_space` from
  `lpFreeBytesAvailable` (caller-scoped, honours per-user quotas),
  and `total_space` is computed from cluster math
  (`sectors_per_cluster * bytes_per_sector * total_number_of_clusters`)
  so it reports volume capacity rather than the caller's quota. On
  quota-enabled volumes the three fields now carry distinct,
  documented meanings; previously `free_space` and `available_space`
  were identical and `total_space` under-reported capacity.
- Added `target_os = "fuchsia"` to the Unix `allocate` `fallocate`
  branch (sync and async). Fuchsia is `cfg(unix)` under rustc and
  `rustix` exposes `fallocate` there, so the previous omission left
  the Fuchsia Unix build without an `allocate` symbol once `FileExt`
  was enabled. With this fix, every fs4 feature builds on Fuchsia
  **except** `fs-err3` and `fs-err3-tokio`; those remain blocked on
  `fs-err v3.3.0` referencing `std::os::unix::fs::chroot`, which
  rustc gates out on `target_os = "fuchsia"`. The fs4 code no longer
  has a Fuchsia gap — the remaining one is upstream
  (<https://github.com/andrewhickman/fs-err/issues/90>).

### Testing

- New regression test `allocate_reserves_blocks_on_sparse_file`
  (sync and async, Linux-gated) creates a sparse file via `set_len`,
  asserts `allocated_size == 0`, calls `allocate`, and asserts
  blocks are reserved.

### Documentation

- README gained a **Minimum Supported Rust Version** section noting
  that the crate's declared MSRV (`rust-version = "1.75.0"`) covers
  the default `sync` feature, and that `async-std` / `smol` inherit
  a higher effective MSRV (1.85) from their transitive dependencies
  (`async-lock`, `async-signal`).

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
  with `Error(io::Error)` and `WouldBlock` variants. Implements
  `Debug`, `Display`, `std::error::Error` (with `source()` exposing
  the inner `io::Error`), `From<io::Error>` (kind `WouldBlock`
  collapses into the `WouldBlock` variant; everything else wraps into
  `Error`), and `From<TryLockError> for io::Error` (`WouldBlock`
  becomes `io::Error::from(io::ErrorKind::WouldBlock)`; `Error(inner)`
  passes through verbatim).

### Fixes

- Fixed feature typos that made the crate fail to compile with only
  `fs-err2`, `fs-err3`, `fs-err2-tokio`, or `fs-err3-tokio` enabled
  (without `sync` / `tokio`). `#[cfg(feature = "fs-err")]` and
  `feature = "fs-err-tokio"` both referenced feature names that do not
  exist in `Cargo.toml`.
- On Windows, skip the internal `set_len` call when the file's existing
  cluster-aligned allocation already covers `len`. Avoids the Windows
  buffered-I/O quirks observed when the end-of-file pointer is moved
  inside an already-allocated cluster (#13). The trait doc now carves
  this behavior out from the general "file size is at least `len`
  bytes" contract.
- Short-circuited `allocate` on Unix when the file is already at least
  `len` bytes long. Fixes macOS `fallocate` spuriously returning
  `ENOSPC` when re-calling `allocate(len)` on an existing file (#15).
- Added `cygwin` to the `allocate` `target_os` set so builds stop
  failing with a missing `sys::allocate` symbol on that target (#44).
- Added `redox` and `cygwin` to the async `allocate` fallback branch
  so it matches the sync variant (#43 follow-up).
- Moved AIX from the async `fallocate` branch to the `set_len`
  fallback branch, matching the sync implementation.
- Trait docs now explicitly state that locks are released
  automatically when the owning file handle is closed (#23).
- Removed the stale trait-doc reference to non-existent cross-platform
  tests in `lib.rs` (#16).
- Mitigated #31 (compiler warning about name collisions with upcoming
  std methods): because 1.0 renames the trait methods to match std
  exactly, `std::fs::File::lock` / `try_lock` / `unlock` (stable in
  Rust 1.89+) now win via inherent dispatch and `unstable_name_collisions`
  no longer fires for std users on recent rustc.
- Gated the `lock_impl!` macro with `#[allow(unused_macros)]` so
  `cargo clippy --no-default-features` (filesystem-stats-only build)
  does not fail under `-D warnings`.
- Removed dead duplicate macros `cfg_fs2_err` / `cfg_fs3_err` from
  `lib.rs` (unified with `cfg_fs_err2` / `cfg_fs_err3`).
- Updated `html_root_url` to the 1.0.0 docs.rs path.

### Dependency updates

- Bumped `windows-sys` from 0.59 to 0.61.

### Testing

- Removed all `#[bench]` functions from the test harness. They
  measured OS syscalls (`flock`, `LockFileEx`, `fallocate`, `statvfs`)
  rather than fs4 code, and produced numbers dominated by the
  underlying filesystem. Dropping them lets the crate build and test
  on **stable** Rust (the bench harness was the only thing pinning
  nightly via `#![cfg_attr(test, feature(test))]`).
  `rust-toolchain.toml` is now `stable`.
- Added regression tests:
  - `allocate_preserves_eof_within_cluster` — exercises the #13
    precondition (cluster-aligned `AllocationSize` with EOF inside
    the cluster) and asserts EOF is not moved when `allocate(len)` is
    re-called with `len <= allocated_size` on Windows.
  - `allocate_idempotent` — exercises the #15 precondition (back-to-back
    `allocate(len)` calls on macOS) to prevent a regression in the
    short-circuit.
  - Seven `TryLockError` unit tests covering variants, `Display`,
    `std::error::Error::source`, `From<io::Error>`, `From<TryLockError>
    for io::Error`, and round-trip preservation of `ErrorKind`.
  - `FsStats` getter + derive unit tests (previously every test
    destructured the struct, so the four getter method bodies were
    never executed and showed as uncovered).
- Fixed a long-standing macOS test failure: the cross-platform
  `filesystem_space` test asserted `available_space <= free_space`,
  which does not hold on macOS APFS. APFS counts purgeable space
  (snapshots, cached data reclaimable on demand) in `f_bavail` but
  not in `f_bfree`, so the POSIX invariant is violated on every run.
  The assertion is removed and the async variant now makes a single
  `statvfs` call plus destructure instead of three separate calls
  racing with concurrent filesystem activity from other tests.

### CI / tooling

- New `.github/workflows/coverage.yml` workflow: per-OS matrix
  (ubuntu-latest, macos-latest, windows-latest) running
  `cargo tarpaulin --engine llvm --all-features --run-types tests
  --ignore-tests` on each runner and uploading per-OS reports to
  Codecov with per-OS flags. Each runner `--exclude-files` the other
  OS's sources so they do not register as 0/N uncovered lines.
- The `cross` job now installs the MinGW-w64 toolchain when targeting
  `*-pc-windows-gnu`. `windows-sys` 0.61 invokes `dlltool` during
  build, and `ubuntu-latest` ships only the x86_64 MinGW toolchain
  by default.
- README install snippets updated from `fs4 = { version = "0.13", ... }`
  to `fs4 = { version = "1", ... }`.
- Housekeeping: added `.codecov.yml`, `rustfmt.toml` (2-space indent,
  explains the session-wide reformat), and
  `.github/workflows/loc.yml`; removed the obsolete `tea.yaml`.

## 0.13.0

### Breakage

- Make `try_lock_*` return `std::io::Result<bool>`, which is compatible with the upcoming `std::fs::File::try_lock*` in `std`.

[`std::fs::File::lock`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.lock
[`std::fs::File::try_lock`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock
[`std::fs::TryLockError`]: https://doc.rust-lang.org/std/fs/enum.TryLockError.html
