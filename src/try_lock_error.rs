use std::fmt;
use std::io;

/// The error returned from [`FileExt::try_lock`](crate::FileExt::try_lock)
/// and [`FileExt::try_lock_shared`](crate::FileExt::try_lock_shared).
///
/// This mirrors [`std::fs::TryLockError`] that was stabilized alongside
/// [`File::try_lock`](std::fs::File::try_lock) in Rust 1.89.
#[derive(Debug)]
pub enum TryLockError {
  /// The lock could not be acquired due to an I/O error on the file.
  Error(io::Error),
  /// The lock could not be acquired at this time because the operation would
  /// otherwise block.
  WouldBlock,
}

impl fmt::Display for TryLockError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Error(err) => err.fmt(f),
      Self::WouldBlock => "try_lock failed because the operation would block".fmt(f),
    }
  }
}

impl std::error::Error for TryLockError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::Error(err) => Some(err),
      Self::WouldBlock => None,
    }
  }
}

impl From<TryLockError> for io::Error {
  fn from(err: TryLockError) -> Self {
    match err {
      TryLockError::Error(err) => err,
      TryLockError::WouldBlock => io::Error::from(io::ErrorKind::WouldBlock),
    }
  }
}

impl From<io::Error> for TryLockError {
  fn from(err: io::Error) -> Self {
    if err.kind() == io::ErrorKind::WouldBlock {
      TryLockError::WouldBlock
    } else {
      TryLockError::Error(err)
    }
  }
}

#[cfg(test)]
mod tests {
  //! Regression tests for the 1.0 `TryLockError` API. These exist so
  //! that the `Result<(), TryLockError>` shape (which now mirrors
  //! `std::fs::TryLockError`) stays compatible with std's semantics.

  use super::*;
  use std::error::Error as _;
  use std::io;

  /// A raw `io::Error` with kind `WouldBlock` must collapse into the
  /// `WouldBlock` variant — callers rely on this to distinguish "lock
  /// contended" from "real I/O failure".
  #[test]
  fn from_io_error_would_block_collapses_to_would_block() {
    let err: TryLockError = io::Error::from(io::ErrorKind::WouldBlock).into();
    assert!(matches!(err, TryLockError::WouldBlock));
  }

  /// Any other `io::Error` kind must surface as `Error(inner)`,
  /// preserving the underlying `io::Error` so callers can inspect it.
  #[test]
  fn from_io_error_other_preserves_inner() {
    let err: TryLockError = io::Error::from(io::ErrorKind::PermissionDenied).into();
    match err {
      TryLockError::Error(inner) => {
        assert_eq!(inner.kind(), io::ErrorKind::PermissionDenied);
      }
      other => panic!("expected TryLockError::Error, got {other:?}"),
    }
  }

  /// `io::Error::from(TryLockError::WouldBlock)` must produce an
  /// error whose `kind()` is `WouldBlock`, matching std's
  /// `impl From<TryLockError> for io::Error`.
  #[test]
  fn into_io_error_would_block() {
    let err: io::Error = TryLockError::WouldBlock.into();
    assert_eq!(err.kind(), io::ErrorKind::WouldBlock);
  }

  /// `io::Error::from(TryLockError::Error(inner))` must return the
  /// original `inner` verbatim (same kind, same OS error if any).
  #[test]
  fn into_io_error_passes_inner_through() {
    let inner = io::Error::from(io::ErrorKind::NotFound);
    let err: io::Error = TryLockError::Error(inner).into();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
  }

  /// Round-trip: `io::Error` → `TryLockError` → `io::Error` must
  /// preserve the `ErrorKind` for both variants.
  #[test]
  fn round_trip_preserves_error_kind() {
    for kind in [
      io::ErrorKind::WouldBlock,
      io::ErrorKind::PermissionDenied,
      io::ErrorKind::NotFound,
      io::ErrorKind::Interrupted,
    ] {
      let original = io::Error::from(kind);
      let converted: io::Error = TryLockError::from(original).into();
      assert_eq!(
        converted.kind(),
        kind,
        "round-trip changed kind for {kind:?}"
      );
    }
  }

  /// `Display` must produce a non-empty message for both variants so
  /// logs and `?` chains remain human-readable.
  #[test]
  fn display_is_non_empty() {
    assert!(!TryLockError::WouldBlock.to_string().is_empty());
    let inner = io::Error::from(io::ErrorKind::NotFound);
    assert!(!TryLockError::Error(inner).to_string().is_empty());
  }

  /// `std::error::Error::source` must expose the inner `io::Error`
  /// for the `Error` variant and return `None` for `WouldBlock`, so
  /// downstream error-chain walkers (anyhow, eyre, etc.) can recover
  /// the OS error.
  #[test]
  fn source_exposes_inner_only_for_error_variant() {
    assert!(TryLockError::WouldBlock.source().is_none());

    let inner = io::Error::from(io::ErrorKind::NotFound);
    let err = TryLockError::Error(inner);
    let src = err.source().expect("Error variant must expose source");
    assert!(
      src.downcast_ref::<io::Error>().is_some(),
      "source must be an io::Error"
    );
  }
}
