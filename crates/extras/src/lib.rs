//! Optional composite Material components for GPUI desktop applications.

#[cfg(feature = "expressive-groups")]
mod groups;
#[cfg(feature = "expressive-groups")]
pub use groups::*;
#[cfg(feature = "search")]
mod search;
#[cfg(feature = "search")]
pub use search::*;
