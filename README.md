[<img alt="crates.io" src="https://img.shields.io/crates/v/windows-helpers.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/windows-helpers)

A library crate to make working with the [`windows` crate](https://crates.io/crates/windows) a bit easier and more pleasant:

- Among other things, it provides methods for conversion of return values into `windows::core::Result`, `ResGuard` to auto-free on drop, `dual_call()` to simplify two-step calls, and bit manipulation functions.
- The `win32_app` module helps quickly getting a simple app off the ground to allow message receiving and tray icons, e.g.

# Feature Requirements

If not instructed to activate more specific features, activate at least the feature that specifies the `windows` crate version you depend on - up until the first non-zero part. An example would be the feature name `windows_v0_52`. Not every version is already available (see [`Cargo.toml`](Cargo.toml) to get an idea of the feature name concept, although not every feature is for public consumption).

If more specific features are needed, their names are specifically specified or built from the previously described feature name, the infix `f` for "feature" and a `windows` crate feature. Example: `windows_v0_52_f_Win32_Foundation`. The `windows` crate features needed depend on the types used and their modules. Note that the `windows` crate may move types to other modules, while the online documentation only reflects the newest crate structure.

# License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
