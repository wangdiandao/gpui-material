//! Single-binary integration tests for the `gpui-material` public facade.
//!
//! Keeping facade integration suites as modules of one test target avoids
//! linking duplicate copies of GPUI and all component crates per file.

#[cfg(feature = "full")]
mod facade_contracts;
mod harness;
#[cfg(feature = "full")]
mod ui;
