# 第三方声明

[中文](#中文) · [English](#english)

## 中文

项目原创代码与文档适用根目录的 [Apache-2.0](LICENSE)。本阶段从以下来源核对技术事实，仓库中的规格与验收文字由本项目编写。

| 来源 | 用途与许可记录 |
| --- | --- |
| [GPUI Kit v0.9.1](https://github.com/longbridge/gpui-kit/tree/v0.9.1) | Base 代码为 Apache-2.0。指定网站文档另有 CC BY 4.0 授权，适用路径、归属和改编说明见[上游文档许可](https://github.com/longbridge/gpui-kit/blob/v0.9.1/LICENSE-DOCS.md)。 |
| [Zed GPUI](https://github.com/zed-industries/zed/tree/d2a4fb6ded7c470be25d1541281da3882495b65b/crates/gpui) | 固定的 gpui-pre 软件包使用 Apache-2.0。 |
| [Google Material Design 3](https://m3.material.io/) | 设计参考。站点文字、图片和品牌素材遵循各自权利声明。 |
| [AndroidX](https://github.com/androidx/androidx)与 [Material Components Android](https://github.com/material-components/material-components-android) | Apache-2.0 源码参考。Android Developers 页面另按其[站点许可](https://developer.android.com/license)处理。 |
| [Material Color Utilities](https://github.com/material-foundation/material-color-utilities)、[material-color-utils](https://github.com/RuurdBijlsma/material-color-utilities-rs) 与 [polymorpher](https://github.com/Aiving/polymorpher) | Apache-2.0 算法参考或候选依赖。移植时保留原有版权、许可证和适用的 NOTICE。 |
| [Material Symbols](https://developers.google.com/fonts/docs/material_symbols) | Apache-2.0 图标候选。纳入资产时记录文件、修改和许可副本。 |

主题的 `dynamic-color` 功能依赖 crates.io 的 [material-color-utils 0.1.3](https://crates.io/crates/material-color-utils/0.1.3)。该包的 LICENSE 为 Apache-2.0。[主题 Token 索引](docs/theme/token-index.md)逐项记录取自 Compose 或固定设计页的数值与引用。

引入第三方源码、图标、字体或文档节选时，在其归属阶段记录精确来源、版本、修改与再分发条款。源版本和 Cargo 候选见[设计依据](docs/design-sources.md)及[架构与依赖](docs/architecture.md)。

## English

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

When including third-party source, icons, fonts, or documentation excerpts, record the exact source, version, modifications, and redistribution terms in the owning phase. [Design sources](docs/design-sources.en.md) · [Architecture and dependencies](docs/architecture.en.md)
