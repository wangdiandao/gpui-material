#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod catalog;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod gallery;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod overview;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod styles_examples;
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub(crate) mod window_chrome;
