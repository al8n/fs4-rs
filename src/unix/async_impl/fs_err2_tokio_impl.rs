use fs_err2::tokio::File;
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::AsRawFd;

lock_impl!(File);
allocate!(File);
allocate_size!(File);

test_mod! {
  tokio::test,
  use fs_err2::tokio as fs;
}
