/// `FsStats` contains some common stats about a file system.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FsStats {
  pub(crate) free_space: u64,
  pub(crate) available_space: u64,
  pub(crate) total_space: u64,
  pub(crate) allocation_granularity: u64,
}

impl FsStats {
  /// Returns the number of free bytes in the file system containing the provided
  /// path.
  pub fn free_space(&self) -> u64 {
    self.free_space
  }

  /// Returns the available space in bytes to non-privileged users in the file
  /// system containing the provided path.
  pub fn available_space(&self) -> u64 {
    self.available_space
  }

  /// Returns the total space in bytes in the file system containing the provided
  /// path.
  pub fn total_space(&self) -> u64 {
    self.total_space
  }

  /// Returns the filesystem's disk space allocation granularity in bytes.
  /// The provided path may be for any file in the filesystem.
  ///
  /// On Posix, this is equivalent to the filesystem's block size.
  /// On Windows, this is equivalent to the filesystem's cluster size.
  pub fn allocation_granularity(&self) -> u64 {
    self.allocation_granularity
  }
}

#[cfg(test)]
mod tests {
  //! The cross-platform `filesystem_space` tests destructure `FsStats`
  //! directly, and the top-level `free_space` / `available_space` /
  //! `total_space` / `allocation_granularity` functions on the crate
  //! root use field access too, so the four getter methods above
  //! never get called from the integration tests. These unit tests
  //! exist so coverage reflects that the getters do what they claim.

  use super::*;

  #[test]
  fn getters_return_fields_verbatim() {
    let stats = FsStats {
      free_space: 1,
      available_space: 2,
      total_space: 3,
      allocation_granularity: 4,
    };
    assert_eq!(stats.free_space(), 1);
    assert_eq!(stats.available_space(), 2);
    assert_eq!(stats.total_space(), 3);
    assert_eq!(stats.allocation_granularity(), 4);
  }

  #[test]
  fn derives_work() {
    let a = FsStats {
      free_space: 1,
      available_space: 2,
      total_space: 3,
      allocation_granularity: 4,
    };
    let b = a.clone();
    assert_eq!(a, b);
    assert!(!format!("{a:?}").is_empty());

    let c = FsStats {
      free_space: 9,
      ..a.clone()
    };
    assert_ne!(a, c);
  }
}
