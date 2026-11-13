# P1-08 Pure algorithm evidence

English | [中文](README.zh.md)

Recorded on 2026-11-13 with Windows 11 x64, Rust/Cargo 1.99.1, and pinned crates.io resolution.

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-theme/Cargo.toml -- --nocapture
cargo +1.99.1 tree --locked --manifest-path experiments/p1-theme/Cargo.toml --edges features
cargo +1.99.1 tree --locked --manifest-path experiments/p1-theme/Cargo.toml --edges features --invert material-color-utils
```

| Check | Expected | Observed | Status |
| --- | --- | --- | --- |
| Blue at tone 40 | `ff343dff` | `ff343dff` | pass |
| Red Tonal Spot, dark, default contrast | `primaryFixed = ffffdad4` | Spec2021 returned `ffffdad4` | pass |
| Fixed roles at default contrast | Light and dark match for fixed roles. Primary changes with appearance. | `primaryFixed` and `primaryFixedDim` matched. `primary` differed. | pass |
| Red at low, default, and high contrast | `primary/onPrimary` contrast increases | Light 4.544/6.429/13.935, dark 4.535/7.750/18.427 | pass |
| Direct call with out-of-range tones | `ratioOfTones(-10, 110) = 21` | Rust 0.1.3 returned 34.20435515298331 | fail |
| Adapted out-of-range tones | 21 within 0.001 | Returned 21 after clamping to [0, 100] at the call boundary | pass |
| chrono date boundaries | Leap day, year rollover, and range bounds | 2024-01-31 plus one month was 2024-02-29. 2023-02-29 was invalid. Out-of-range max/min dates returned None. | pass |
| Pure dependency graph | Excludes GPUI. MCU features exclude image, serde, and rayon. | The reverse MCU feature graph showed only the probe. Package graph included only Rust algorithm and platform fundamentals. | pass |

The pinned [official out-of-range vector][vector] and [clamping semantics][clamp] confirm the discrepancy. The minimal reproduction is `Contrast::ratio_of_tones(-10., 110.)`. When upgrading to an upstream release with equivalent clamping, rerun the vectors before removing the adapter. This evidence covers candidate feasibility.

The chrono date boundaries come from the [P1 acceptance commit](https://github.com/wangdiandao/gpui-material/tree/b0250f5f120ab2fbadfe41b5529612e984358295). The current probe retains the color vectors. The host's `theme` probe checks Base Calendar's disabled leap-day behavior.

[vector]: https://github.com/material-foundation/material-color-utilities/blob/194144d75ed7d34f06264a625e086db81a7ab9aa/dart/test/contrast_test.dart#L15-L18
[clamp]: https://github.com/material-foundation/material-color-utilities/blob/194144d75ed7d34f06264a625e086db81a7ab9aa/typescript/contrast/contrast.ts#L36-L47
