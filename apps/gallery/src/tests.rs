#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
mod desktop_tests;
mod drawing;
mod input_examples;
#[cfg(not(target_arch = "wasm32"))]
mod perf_bench;
mod scrim_tests;
mod swipe_to_dismiss_tests;
mod theme_tests;
