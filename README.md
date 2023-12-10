[<img alt="crates.io" src="https://img.shields.io/crates/v/windows-helpers.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/windows-helpers)

A library crate to make working with the [`windows` crate](https://crates.io/crates/windows) a bit easier and more pleasant.

# Development Notes

- When using this crate in another crate using a `path = "..."` dependency and, in the other crate, you're using an older `windows` crate version than the newest (than permitted by this crate's `Cargo.toml`), Cargo seems to erroneously pick this crate's `windows` crate version when this crate's `Cargo.lock` still exists. This can lead to strange compiler errors like `` expected `CopyType`, found `ReferenceType` ``, `` `?` couldn't convert the error to `windows::core::Error` `` or `` the trait `windows_core::param::CanInto<windows::Win32::Foundation::HANDLE>` is not implemented for `HANDLE` ``. This crate's `Cargo.lock` has to be deleted before proceeding.

# License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
