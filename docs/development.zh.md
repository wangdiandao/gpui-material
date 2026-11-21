# 开发与验证

[English](development.md) | 中文

<a id="contribution-workflow-and-branch-naming"></a>

## 贡献流程与分支命名

- Issue 模板、设计变更与评审要求见[贡献指南](../CONTRIBUTING.zh.md)。
- 从最新 `main` 建分支，使用 `<类型>/<简短主题>`，如 `ci/cross-platform-workflow`。主题使用小写英文和连字符。在 PR 说明里关联 Issue。
- 按改动目的选前缀。`feat/` 新功能，`fix/` 缺陷，`docs/` 文档，`test/` 测试，`refactor/` 重构，`ci/` CI，`chore/` 工程维护，`release/` 发布。例如 `fix/pointer-capture`。

<a id="workspace-and-component-development"></a>

## Workspace 与组件开发

- 生产库放在 `crates/`，组件按族组织。Gallery 位于 `apps/gallery/`，其中的示例调用生产库的公开 API。[架构与依赖](architecture.zh.md) · [组件](component.zh.md)
- 组件改动在所属 crate 验证公共行为，并从 Gallery 复测相关变体和交互。可独立复现的场景使用最小入口。`xtask/` 存放工程检查和离线生成。

<a id="coding-conventions"></a>

## 编码规范

- 从[文档索引](index.zh.md)定位当前规格。修改 Material 呈现时遵循[锁定规范与评审规则](../CONTRIBUTING.zh.md#设计规范)。
- 变更需要修改文档时，先完成文档修改，再开始实现。相关文档与代码可以放在同一次提交和同一个 PR 中。
- 英文 `foo.md`、中文 `foo.zh.md` 与逐节指纹 `foo.i18n.yaml` 放在同一目录。标题下链接另一语言，正文链接优先指向同语言页面。在同一 PR 同步两页，确认内容对应后运行 `cargo xtask docs --write path/to/foo.md` 更新记录。`cargo xtask check` 核验配对、语言入口、标题层级、代码示例和指纹。GitHub Issue 和 PR 模板使用英文。[DeepSeek Harness 文档约定](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/i18n/README.md)
- theme 保持纯 Rust，foundation 负责 GPUI/Base 适配。应用或 Base 持有业务与输入状态，Material 持有呈现和新增组合状态。优先复用现有公共接口。[架构与依赖](architecture.zh.md)
- 遵循高内聚、低耦合的模块架构规范，以长期可维护性为首要考量。按职责边界（如公开接口、状态流转、几何与布局计算、样式解析、视图呈现与测试）组织模块和函数，而不是按机械的代码行数拆分。
- 当单个模块混杂多个正交职责、同级依赖出现耦合，或单个 `render` / 状态流转函数同时承担状态准备、交互绑定与多段布局拼装时，拆分为职责单一的子模块或辅助函数。拆分时保持公开导出路径稳定，内部项收窄为 `pub(super)` 或 `pub(crate)`，同级子模块保持单向窄依赖；较大规模测试抽离到独立的 `#[cfg(test)]` 子模块（含 Gallery 的 `*_tests.rs`），自动生成的数据表保持整体不手拆。
- 采用 Rust 惯用命名和稳定的语义标识。注释说明不变量、生命周期与平台差异。公开文档交代状态所有权、默认值、焦点、回调和平台支持。

<a id="testing-conventions"></a>

## 测试规范

- 每次变更都验证受影响的行为。先用已有测试、构建检查或最小复现。规格要求、公共行为或关键不变量缺少覆盖，或可稳定复现的缺陷仍有实际复发风险时，新增长期自动化测试。仅改文档、格式或内部结构且已有覆盖足够时，运行相关检查即可。
- 在最低有效层验证可观察结果。同一行为优先补到已有用例，只有上层存在独有风险才增加 UI、集成或原生验证。用例应可重复运行，并能说明覆盖的契约和退化时的失败表现。组件按适用场景验证真实渲染、键盘、指针、语义、焦点、取消和原生系统路径。[平台与质量要求](verification.zh.md)
- 单次修复验证、环境排查、迁移和能力探测可用临时命令或脚本。重要结果和复现步骤写入对应 Issue 或 PR。只有覆盖仍受支持且现有测试漏掉的回归时，才将最小断言纳入长期套件。
- 修改行为或替换探针时，清理已被正式测试覆盖、失去验证对象或仅用于一次排查的用例及辅助代码。删除前核对独有覆盖与引用。

<a id="upstream-contributions"></a>

## 上游贡献

- 确认通用上游缺陷时，先写最小复现并检查现有 Issue 和 PR。遵守上游贡献、测试、许可与评审规则。
- 补丁聚焦通用缺口并附最小验证。用项目 Issue 跟踪上游评审、合并、发布和临时适配的退出条件。
- 更新固定依赖前验证包身份、平台回归和影响范围。以对应上游发布记录核验版本、提交与源码。

<a id="development-environment"></a>

## 开发环境

安装 [Rustup](https://rustup.rs/)，从仓库根目录运行 Cargo。[rust-toolchain.toml](../rust-toolchain.toml) 固定仓库开发与 CI 使用的 Rust 编译器、rustfmt 与 Clippy；发布 crate 在 [Cargo.toml](../Cargo.toml) 中声明的最低支持 Rust 版本（MSRV）为 1.95.0。

| 平台 | 构建依赖 |
| --- | --- |
| Windows 11 x64 | Visual Studio Build Tools 的「使用 C++ 的桌面开发」工作负载，包含 MSVC x64 和 Windows SDK。 |
| Apple Silicon macOS | Xcode 命令行工具。 |
| Ubuntu 24.04 x64 | 见 [Engineering CI](../.github/workflows/ci.yml) 的 `Install Linux platform dependencies` 步骤。 |

用 `rustc -vV` 核对工具链和宿主平台。启动 Gallery 时，可以传入示例名称或 ID 直接打开该示例。

```powershell
cargo gallery
cargo gallery text-field
```

工作区默认成员为主题 crate 和 `xtask`。用 `-p gpui-material-gallery` 显式选择 Gallery。在 worktree 中将 `CARGO_TARGET_DIR` 指向主检出的 `target` 以复用开发构建。

<a id="rendering-performance"></a>

## 性能检查

日常检查使用开发配置，性能对比使用锁定依赖的 Release 构建。打开 Button 示例，分别检查静止、悬停、按下和释放。用 `--reduce-motion` 做诊断对照。

```powershell
cargo run --locked --release -p gpui-material-gallery -- button
cargo run --locked --release -p gpui-material-gallery --features frame-profile -- button --frame-profile
cargo test --locked --release -p gpui-material-foundation --lib ripple_clip_benchmark -- --ignored --nocapture
cargo test --locked --release -p gpui-material-gallery --features test-support --lib perf_bench -- --ignored --nocapture --test-threads=1
```

在同一台机器上对改动前后各独立运行至少五轮相同的 Ripple 微基准。如果基线提交早于基准测试，仅将基准测试移植到该提交。基准测量预构建网格的裁剪时间，不包含形状构建和屏幕呈现。在 Issue 或 PR 记录提交、负载、耗时波动和相关实窗检查。测量原生帧时，分别报告绘制 CPU 耗时和屏幕呈现间隔。

`perf_bench` Release 基准在固定 `1040 × 744` 视口下执行 20 帧预热 + 100 帧采样（`16 ms` 时钟步进），按三个独立归因层次拆解开销，避免误判回归来源：

- **引擎层（`engine/*`）**：空 GPUI 窗口、64 个裸 `div`/文本图元、200 行裸 GPUI `list`，配合 Foundation 的 `ripple_clip_benchmark`。
- **组件层（`component/*`）**：直接挂载在 `MaterialRoot` 下、不含 Gallery 外壳的独立 Material 组件（64 按钮网格静止/交互、填充/描边/多行 `TextField` 静止/输入、200 行复杂 `ListItem` 虚拟列表静止/滚动、`Scaffold`+`AppBar`+`NavigationRail`+`Tabs`+`NavigationBar` 静止/切换，以及 `LoadingIndicator`+波浪 `LinearProgress`/`CircularProgress` 可见动画、屏外裁剪暂停与减少动态效果）。
- **示例页与 Gallery 外壳层（`story/*` 与 `gallery/*`）**：对比独立 `Story::new_view` 与完整 `Gallery::new`（侧边目录、搜索框、工具栏与 `ListDetail` 外壳），量化 Gallery 自身开销。

参考 Release 基线（`b3a538e`，Windows 11 x64，AMD Ryzen 9 8945HX，`rustc 1.99.1`，`--locked --release`，`perf_bench` 取 3 轮中位 p50/p95，`ripple_clip_benchmark` 取 5 轮中位数）：

| 基准项 | 层次 | p50 (µs) | p95 (µs) | Quads | 调度帧数 |
| --- | --- | ---: | ---: | ---: | ---: |
| `engine/empty_window` | Engine | 7.9 | 8.6 | 0 | 0 |
| `engine/raw_quads_64` | Engine | 295.9 | 375.8 | 64 | 0 |
| `engine/raw_virtual_list_200` | Engine | 244.4 | 327.5 | 59 | 0 |
| `component/button_grid_64_idle` | Component | 2497.9 | 3353.5 | 112 | 0 |
| `component/button_grid_64_active` | Component | 2943.2 | 4711.8 | 112 | 1448 |
| `component/text_field_idle` | Component | 272.6 | 355.1 | 19 | 0 |
| `component/text_field_typing` | Component | 277.5 | 408.9 | 20 | 0 |
| `component/complex_list_200_idle` | Component | 1903.0 | 3591.2 | 40 | 0 |
| `component/complex_list_200_scroll` | Component | 1956.3 | 2570.2 | 41 | 244 |
| `component/navigation_scaffold_idle` | Component | 552.1 | 692.8 | 11 | 0 |
| `component/navigation_rail_transition` | Component | 574.9 | 713.7 | 11 | 263 |
| `component/animated_visible` | Component | 85.6 | 137.2 | 9 | 297 |
| `component/animated_clipped_offscreen` | Component | 27.6 | 51.3 | 1 | 0 |
| `component/animated_reduced_motion` | Component | 96.1 | 142.4 | 10 | 0 |
| `story/button` / `gallery/button` | Story / Gallery | 964.6 / 4903.2 | 1137.9 / 6249.2 | 35 / 47 | 0 / 0 |
| `story/text_field` / `gallery/text_field` | Story / Gallery | 752.7 / 4899.2 | 894.0 / 6317.1 | 54 / 64 | 0 / 0 |
| `story/list` / `gallery/list` | Story / Gallery | 4909.5 / 12027.4 | 6593.5 / 15308.4 | 28 / 45 | 0 / 0 |
| `story/navigation_rail` / `gallery/navigation_rail` | Story / Gallery | 657.0 / 5470.8 | 806.2 / 7908.8 | 22 / 47 | 0 / 0 |
| `story/loading_indicator` / `gallery/loading_indicator` | Story / Gallery | 676.5 / 4723.2 | 1011.4 / 7850.4 | 37 / 49 | 198 / 198 |

`ripple_clip_benchmark`（5 轮中位数）：`capsule expanding` 8,445.6 ns/clip（37 三角形）、`capsule covered` 20,071.0 ns/clip、`morph expanding` 11,330.8 ns/clip（40 三角形）、`morph covered` 21,587.1 ns/clip。该基线表明，Gallery 外壳（42 项侧边目录、搜索栏、工具栏与 `ListDetail` 框架）独立占用了约 3.9–7.1 ms 的 p50 帧耗时，与被测组件本身开销分离清晰。

可选的单窗口帧采样预热两秒后打印开始标记，采样十秒并报告绘制次数、CPU 绘制耗时和活跃帧提交间隔的 p50/p95/p99。分别采集静止、鼠标横扫和重复按压场景，改动前后使用同样的输入序列。提交间隔以 GPUI 向平台提交帧的时间为准。

<a id="local-checks"></a>

## 本地检查

检查文档和仓库时，先暂存新增、重命名和删除的 Markdown，使 `xtask` 能从 Git 索引发现这些变更。检查器校验本地链接、标题锚点、双语配对、Token 生成物，以及 Windows、macOS、Linux 的 GPUI/Base 依赖图。

```powershell
git diff --check origin/main
cargo xtask check
```

修改 Rust 代码时，检查格式并运行受影响的测试。下面的命令对应 CI 的全工作区 lint 和测试范围。

```powershell
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

单独运行 Gallery UI 测试可用 `cargo test --locked -p gpui-material-gallery --features test-support`。修改 Token 数据时按[生成步骤](theme/theme.zh.md#token-data)操作。

<a id="documentation-website"></a>

## 文档网站

[网站源码](../website/)使用 Astro 和 [MDUI](https://www.mdui.org/zh-cn/) Web Components，通过 Pagefind 为静态页面建立搜索索引。安装 Node.js 24 和 npm 后，从仓库根目录执行以下命令。

```sh
npm --prefix website ci
npm --prefix website run dev
npm --prefix website run build
npm --prefix website run preview
```

开发服务器使用 `/`。生产构建默认使用 `/gpui-material/` 并输出到 `website/dist/`。搜索索引可在生产预览中使用。

共用指南在 `docs/` 编辑，控件示例在 `website/component/` 编辑。同时补充英文和中文说明，同一个控件页面的中英文共用一份 Rust 示例。

网站组件目录和侧栏共用基础、表单、布局和高级组件四组，侧栏在各组内按 Rust 名称排序。导入语句和用法代码分开编写，可交互的浏览器预览放在「示例」下。

网站导航和界面控件使用 MDUI。浏览器预览把 Rust Gallery 编译为 WebAssembly，每个演示在各自的 GPUI 画布中运行。画布尺寸与嵌入框一致，由 GPUI 滚动容器处理滚轮和滚动条拖动。浏览器需支持 WebGPU 或 WebGL2。桌面平台专属行为通过原生 Gallery 验证。

在本地使用预览时，先安装网页工具链和匹配的绑定生成器，然后在启动 Astro 前构建 WASM 资源。

```sh
rustup toolchain install nightly-2026-10-31 --profile minimal --target wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
npm --prefix website run build:wasm
```

本地构建使用 Cargo dev profile。Pages 工作流会构建并随网站发布优化后的 WASM。

在 **Settings → Pages → Build and deployment → Source** 选择 **GitHub Actions**。修改网站源码或 WASM 构建输入的 PR 会构建网站。网站、构建输入或文档在 `main` 更新后，会触发 Pages 构建和部署。需要重新部署时，在 Actions 页面选择 `main` 运行 **Deploy website**。

<a id="cold-and-warm-build-baseline"></a>

## 冷热构建基线

```powershell
cargo fetch --locked --target x86_64-pc-windows-msvc
cargo xtask timings target/timings/gallery-baseline
```

每次传入新的报告目录。Cargo 的注册表和源码缓存已预取，冷检查与冷构建各使用自己的空目标目录。随后用同一命令与同一目录测热结果。工具保存四次 wall time、HTML 时间线、Git 提交、目标和工具链。时间线中的 Gallery 最后一个编译单元包含代码生成和链接，可用于定位构建成本。测量使用普通 Gallery feature 和开发 profile，沿用 Cargo 默认作业数。在 Issue 或 PR 中记录关键计时、复现命令和相关运行环境。

<a id="ci-and-merging"></a>

## 持续集成与合并

[Engineering CI](../.github/workflows/ci.yml) 对面向 `main` 的 PR 和 `main` 的 push，将检出内容与事件的基线提交比较，以选择检查作业。手动运行会执行完整检查。

| 作业 | 检查 |
| --- | --- |
| `Change scope` | 判断变更范围，并在有基线提交时检查差异空白。 |
| `Documentation checks` | 文档和工程变更运行 `cargo xtask docs --check`，核验 Markdown 链接与双语配对。 |
| `Website / Build documentation` | 修改网站源码或构建输入的 PR 会构建带 WASM Gallery 的网站并检查链接。Pages 工作流在 `main` 复用此构建。 |
| `Repository checks` | 工程变更检查 Rust 格式、Token 生成物、三平台依赖图及默认工作区。 |
| `Windows MSVC`、`macOS ARM64`、`Linux x64` | 工程变更运行 Clippy、工作区测试、Gallery 无头 UI 测试和原生 Gallery 构建。平台失败后其余平台继续执行。 |
| `CI required` | 核验选中的作业全部通过，其余作业均已跳过。 |

纯文档 PR 运行文档检查，纯网站 PR 运行网站构建，同时修改两处时执行两项检查。Gallery、库和工作区清单等共享 Rust 构建输入有变更时，还会运行原生检查。仅用于浏览器的 Gallery 入口属于网站范围。

Rust 构建缓存按平台隔离，由 `main` 的成功 push 更新，PR 可复用。

发布验收按 [Windows 实窗检查](verification.zh.md#windows-gallery-acceptance)执行。打包验证、crates.io 发布、打 tag 与 GitHub Release 发布通过手动触发的 [Release 工作流](../.github/workflows/release.yml)执行，流程与配置见[版本与发布](release.zh.md#prepare-and-publish)。

GitHub 规则集要求通过 PR 更新 `main`。PR 必须基于最新主线、解决评审对话，并通过 GitHub Actions 的 `CI required` 检查。
