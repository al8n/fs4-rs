macro_rules! allocate_size {
    ($file:ty) => {
        pub fn allocated_size(file: &$file) -> std::io::Result<u64> {
            file.metadata().map(|m| m.blocks() * 512)
        }
    };
}

macro_rules! allocate {
    ($file:ty) => {
        #[cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "android",
            target_os = "emscripten",
            target_os = "nacl",
            target_os = "macos",
            target_os = "ios",
            target_os = "watchos",
            target_os = "tvos"
        ))]
        pub fn allocate(file: &$file, len: u64) -> std::io::Result<()> {
            use rustix::{
                fd::BorrowedFd,
                fs::{fallocate, FallocateFlags},
            };
            unsafe {
                let borrowed_fd = BorrowedFd::borrow_raw(file.as_raw_fd());
                match fallocate(borrowed_fd, FallocateFlags::empty(), 0, len) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(std::io::Error::from_raw_os_error(e.raw_os_error())),
                }
            }
        }

        #[cfg(any(
            target_os = "aix",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "dragonfly",
            target_os = "solaris",
            target_os = "illumos",
            target_os = "haiku",
        ))]
        pub fn allocate(file: &$file, len: u64) -> std::io::Result<()> {
            // No file allocation API available, just set the length if necessary.
            if len > file.metadata()?.len() as u64 {
                file.set_len(len)
            } else {
                Ok(())
            }
        }
    };
}

cfg_sync! {
    pub(crate) mod std_impl;
}

cfg_sync! {
    pub(crate) mod fs_err_impl;
}
