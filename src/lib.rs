mod dual_call;
pub mod error;
mod res_guard;

pub use dual_call::dual_call;
pub use dual_call::FirstCallExpectation;
pub use res_guard::ResGuard;

#[cfg(feature = "windows_v0_48")]
pub(crate) use windows_v0_48 as windows;
#[cfg(feature = "windows_v0_52")]
pub(crate) use windows_v0_52 as windows;
