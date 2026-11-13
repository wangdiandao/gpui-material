# Third-party notices

English | [中文](THIRD_PARTY_NOTICES.zh.md)

Original project code and documentation use the root [Apache-2.0 license](LICENSE). The sources below were consulted to verify technical facts. This project authored its own specifications and acceptance records.

| Source | Use and licensing |
| --- | --- |
| [GPUI Kit v0.9.1](https://github.com/longbridge/gpui-kit/tree/v0.9.1) | Base code is Apache-2.0. Designated website documentation is CC BY 4.0. The [upstream documentation license](https://github.com/longbridge/gpui-kit/blob/v0.9.1/LICENSE-DOCS.md) lists covered paths and attribution and adaptation terms. |
| [Zed GPUI](https://github.com/zed-industries/zed/tree/d2a4fb6ded7c470be25d1541281da3882495b65b/crates/gpui) | The pinned gpui-pre packages use Apache-2.0. |
| [Google Material Design 3](https://m3.material.io/) | Design reference. Site text, images, and branding follow their respective rights notices. |
| [AndroidX](https://github.com/androidx/androidx) and [Material Components Android](https://github.com/material-components/material-components-android) | Apache-2.0 source references. Android Developers pages follow the [site license](https://developer.android.com/license). |
| [Material Color Utilities](https://github.com/material-foundation/material-color-utilities), [material-color-utils](https://github.com/RuurdBijlsma/material-color-utilities-rs), and [polymorpher](https://github.com/Aiving/polymorpher) | Apache-2.0 algorithm references or dependency candidates. Retain original copyrights, licenses, and applicable NOTICE files when porting. |
| [Material Symbols](https://developers.google.com/fonts/docs/material_symbols) | Apache-2.0 icon candidate. Record files, modifications, and license copies when including assets. |

The theme's `dynamic-color` feature uses [material-color-utils 0.1.3](https://crates.io/crates/material-color-utils/0.1.3) from crates.io. The package includes an Apache-2.0 license. The [theme token index](docs/theme/token-index.md) records values and references from Compose or pinned design pages.

The Foundation crate uses [polymorpher 0.1.4](https://crates.io/crates/polymorpher/0.1.4) for rounded polygon geometry. It uses the crate's empty default feature set and depends directly on euclid through polymorpher. The package includes the Apache-2.0 license.

When including third-party source, icons, fonts, or documentation excerpts, record the exact source, version, modifications, and redistribution terms in the owning phase. [Design sources](docs/design-sources.md) · [Architecture and dependencies](docs/architecture.md)
