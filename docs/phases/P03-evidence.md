# P3 验收记录

本地环境为 Windows 11 x64（build 26200）、Rust/Cargo 1.99.1、MSVC。纯主题和 xtask 共用主检出的开发构建目录。PR 关联 [P3-02](https://github.com/wangdiandao/gpui-material/issues/75)，托管结果在 PR 的 Windows MSVC 检查中核对。

| 检查 | 固定来源、期望和观察 | 状态 |
| --- | --- | --- |
| 来源 | Compose common 1.5.0-rc01 源包 SHA-256 为 `38e0515354240bf1e15b6e8891bc02608973f08dc0b8da8cd87a51dfe9ab1031`。AndroidX graphics-shapes 1.1.0 为 `50586aa6c7b3a542a51b2abaf659f7a6c33d8f1a57aaf518dcb74f3d`。Breakpoints 快照与[冻结索引](../material-snapshot.md)相符。 | pass |
| Token | 555 项，覆盖 Reference、System、Component。离线生成的 Rust 和 Markdown 与提交产物一致。类型错配、非法层级、缺失、重复及循环均被拒绝。组件样本保留 Filled Button 的 `Primary` 角色、hover level 1 和 Small Button 的 pressed 形状。 | pass |
| 静态色与层级 | 基准 primary 浅色 `ff6750a4`、深色 `ffd0bcff`。Level 1 叠色为 `fff6eefa`、`ff1e1b24`，按 Compose 的对数公式计算并按 8 位 sRGB 四舍五入。 | pass |
| 源色与语义角色 | MCU revision `194144d75ed7d34f06264a625e086db81a7ab9aa` 的 TypeScript Tonal Spot Spec2021。7 个源色 × 2 种外观 × 4 档对比度（−1、0、0.5、1）× 49 个角色，2,744 个 ARGB 结果精确相等。红色深色默认 `primaryFixed` 为 `ffffdad4`。 | pass |
| HCT 与色板 | 7 个源色的 HCT 数值绝对误差低于 `1e-6`，重建 ARGB 精确相等。蓝色 tone 40 为 `ff343dff`。P1 直接调用 Rust 0.1.3 的越界对比度为 34.204355，适配夹取后为 21，与固定官方实现一致。 | pass |
| 排版 | 15 个 baseline 和 15 个 emphasized 角色，按固定 `TypeScaleTokens.kt` 核对。样本 Display Large 为 57/64 sp、tracking −0.2、400。Emphasized Body Large 为 16/24 sp、tracking 0.15、500。验证字体覆盖、两倍缩放和非法比例回退。 | pass |
| 形状与动效 | Square 宽高 1、圆角 0.3，Triangle 为 3 顶点、半径 1、圆角 0.2、旋转 −90°，Circle 为 10 顶点、圆半径 1。角形验证逻辑起止和窄区域限制。两套动效的 12 组弹簧与固定 Compose 值相同，减少动态效果时立即完成。 | pass |
| 布局 | 验证 600、840、1200、1600 各阈值及其下方、RTL 映射、密度与 48 逻辑单位交互下限。 | pass |
| 依赖 | 三个平台的默认 Workspace 和 theme 各 feature 组合均检查 normal/build/dev 图。主题默认生产图只有自身。MCU 仅在 `dynamic-color` 启用，其自身 feature 为空，`serde` 独立启用。 | pass |

## 复现

```powershell
$env:CARGO_NET_OFFLINE = 'true'
cargo xtask tokens
cargo xtask tokens --check
Remove-Item Env:CARGO_NET_OFFLINE
cargo test --locked -p gpui-material-theme --no-default-features
cargo test --locked -p gpui-material-theme --features dynamic-color
cargo test --locked -p gpui-material-theme --features serde
cargo test --locked --workspace --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
cargo xtask check origin/main
```

[官方向量](../../crates/theme/tests/mcu-spec-2021.json)以固定 MCU 的 `Hct.fromInt`、`Hct.from`、`TonalPalette.fromInt`、`Contrast.ratioOfTones` 与 `SchemeTonalSpot` 计算。使用 TypeScript 5.9.3 将上游 `typescript/index.ts` 编译为 ES2022 模块，在 Node.js 24.14.1 运行。解析时为上游省略扩展名的相对导入补齐 `.js`。每行主题记录依次为源色、isDark、对比度、按角色名排序的 ARGB 列表，构造参数为 `(Hct.fromInt(seed), isDark, contrast, '2021', 'phone')`。HCT 行依次为源色、hue、chroma、tone 和重建 ARGB。运行仓库测试无需 Node.js 或上游源码。

[主题规格](../theme/theme.md)记录桌面策略。绘制、字形度量、动效调度及系统集成在各自的实现阶段验收。

## 依赖准入

`material-color-utils =0.1.3` 对应 `d39ff59a1fca34bdc60e124de2f9f7541fff3c39`，包校验值为 `675994c58762c15c884efab42998c09209f2c451be302e839222fa9e805363fa`。包内 LICENSE 为 Apache-2.0。GitHub 项目未归档，截至核验日最后推送为 2026-03-04。更换实现时重跑固定向量和图检查。

Windows 生产图中直接依赖为 bon 3.10.3、indexmap 2.14.2、papaya 0.2.5、thiserror 2.0.21。传递依赖包括 bon 的构建宏、indexmap 的 hashbrown、papaya 的 seize 和 Windows 平台依赖。下列命令可查看精确的传递版本和启用 feature。

```powershell
cargo tree --locked -p gpui-material-theme --no-default-features --features dynamic-color --edges normal,build
cargo tree --locked -p gpui-material-theme --features dynamic-color --edges features --invert material-color-utils
```
