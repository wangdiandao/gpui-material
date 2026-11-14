//! Optional composite Material components for GPUI desktop applications.

#[cfg(feature = "expressive-groups")]
mod groups;
#[cfg(feature = "expressive-groups")]
pub use groups::*;
