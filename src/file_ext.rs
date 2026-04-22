#[cfg(any(feature = "sync", feature = "fs-err2", feature = "fs-err3"))]
pub(crate) mod sync_impl;

cfg_async!(
  pub(crate) mod async_impl;
);
