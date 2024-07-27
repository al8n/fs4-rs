use fs_err::File;
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::AsRawFd;

lock_impl!(File);
allocate!(File);
allocate_size!(File);

test_mod! {
  use crate::fs_err::FileExt;
  use fs_err as fs;
}
