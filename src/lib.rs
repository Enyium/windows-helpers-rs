mod dual_call;
pub mod error;
mod res_guard;

pub use dual_call::dual_call;
pub use dual_call::FirstCallExpectation;
pub use res_guard::ResGuard;
