pub mod bit_manipulation;
pub mod core;
pub mod foundation;
pub mod win32_app;
pub mod wnds_and_msging;

mod dual_call;
mod empty;
mod res_guard;

pub use dual_call::*;
pub use empty::*;
pub use res_guard::*;

#[cfg(feature = "windows_v0_48")]
pub(crate) use windows_v0_48 as windows;
#[cfg(feature = "windows_v0_52")]
pub(crate) use windows_v0_52 as windows;

pub(crate) mod util; // Temporary.
