# Releases

## 0.13.0

- Make `try_lock_*` returns `std::io::Result<bool>`, which is compatible with the incoming `std::fs::File::try_lock*`.
