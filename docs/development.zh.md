# 开发指南

[English](development.md) | 中文

<a id="contribution-workflow-and-branch-naming"></a>

## 贡献流程与分支命名

- Issue 模板、设计变更与评审要求见[贡献指南](../CONTRIBUTING.zh.md)。
- 从最新 `main` 创建分支，使用小写英文与连字符的 `<类型>/<简短主题>`（如 `feat/split-button`、`fix/pointer-capture`），并在 PR 说明中关联 Issue。
- 按改动目的选择前缀：`feat/`、`fix/`、`docs/`、`test/`、`refactor/`、`ci/`、`chore/` 或 `release/`。

<a id="workspace-and-component-development"></a>

## Workspace 与组件开发

- 生产库位于 `crates/`，按组件族组织；Gallery 位于 `apps/gallery/`，通过示例调用公开 API。[架构与依赖](architecture.zh.md) · [组件](component.zh.md)
- 工程检查和离线生成工具放在 `xtask/`。

<a id="coding-conventions"></a>

## 编码规范

- 从[文档索引](index.zh.md)定位当前规格；修改 Material 呈现时遵循[设计规范规则](../CONTRIBUTING.zh.md#设计规范)。
- 需要同步更新文档时，在同一次 PR 中完成文档与代码改动。
- 英文 `foo.md`、中文 `foo.zh.md` 与逐节指纹 `foo.i18n.yaml` 放在同一目录。标题下链接另一语言，正文链接优先指向同语言页面。更新双语内容后运行 `cargo xtask docs --write path/to/foo.md` 刷新记录。GitHub Issue 和 PR 模板使用英文。[DeepSeek Harness 文档约定](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/i18n/README.md)
- `theme` 保持纯 Rust，`foundation` 负责 GPUI/Base 适配。应用或 Base 持有业务与输入状态，Material 持有呈现和新增组合状态。[架构与依赖](architecture.zh.md)
- 按职责边界（公开接口、状态流转、几何与布局计算、样式解析、视图呈现、测试）组织模块和函数，而不是按机械行数拆分。保持公开导出路径稳定，内部可见性收窄为 `pub(super)` / `pub(crate)`，同级子模块保持单向窄依赖；较大规模测试放入独立的 `#[cfg(test)]` 子模块（含 Gallery 的 `*_tests.rs`），自动生成的数据表保持整体不手拆。
- 采用 Rust 惯用命名和稳定的语义标识。公开 API 文档交代状态所有权、默认值、焦点、回调和平台差异。

<a id="testing-conventions"></a>

## 测试规范

- 新增公开行为、修复可复现缺陷或补齐未覆盖的不变量时，新增或扩展自动化测试。测试应可重复运行，并放在最低有效层（`crates/` 下的单元测试、`apps/gallery/src/` 下的无头 `#[gpui::test]`）。
- 按适用场景覆盖键盘与指针输入、焦点、禁用/错误状态、取消路径、RTL 和减少动态效果。
- 行为变更或覆盖被替代时，及时清理过时的测试与辅助代码。

<a id="upstream-contributions"></a>

## 上游贡献

- 确认上游缺陷时先构造最小复现，检查已有 Issue 和 PR，并遵守上游贡献与许可规则。
- 补丁聚焦通用缺陷，并在本地 Issue 中跟踪上游评审、发布以及临时适配的移除条件。
- 升级固定依赖前，对照上游发布记录核对版本、提交与包身份。

<a id="development-environment"></a>

## 开发环境

安装 [Rustup](https://rustup.rs/)，从仓库根目录运行 Cargo。[rust-toolchain.toml](../rust-toolchain.toml) 固定仓库工具链；发布 crate 在 [Cargo.toml](../Cargo.toml) 中声明的 MSRV 为 1.95.0。

| 平台 | 构建依赖 |
| --- | --- |
| Windows 11 x64 | Visual Studio Build Tools 的「使用 C++ 的桌面开发」工作负载，包含 MSVC x64 和 Windows SDK。 |
| Apple Silicon macOS | Xcode 命令行工具。 |
| Ubuntu 24.04 x64 | 见 [Engineering CI](../.github/workflows/ci.yml) 的 `Install Linux platform dependencies` 步骤。 |

从仓库根目录启动 Gallery，可传入示例名称或 ID 直接打开对应示例：

```powershell
cargo gallery
cargo gallery text-field
```

工作区默认成员为 `gpui-material-theme` 和 `xtask`，用 `-p gpui-material-gallery` 可直接指定 Gallery。在 worktree 中将 `CARGO_TARGET_DIR` 指向主检出的 `target` 目录以复用增量构建。

<a id="local-checks"></a>

## 本地检查

运行 `xtask` 前先暂存新增、重命名和删除的 Markdown 文件，使其进入 Git 索引。`cargo xtask check` 检查 Markdown 链接、标题锚点、双语配对、Token 生成物以及三平台 GPUI/Base 依赖图。

```powershell
git diff --check origin/main
cargo xtask check
```

修改 Rust 代码时运行格式、Clippy、Rustdoc 与测试检查：

```powershell
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
$env:RUSTDOCFLAGS = '-D warnings'; cargo doc --locked --workspace --no-deps --all-features
cargo test --locked --workspace --all-features
```

单独运行 Gallery UI 测试可用 `cargo test --locked -p gpui-material-gallery --features test-support`。修改 Token 数据时按[生成步骤](theme/theme.zh.md#token-data)操作。

<a id="performance-and-build-tools"></a>

## 性能与构建工具

渲染性能对比使用锁定依赖的 Release 构建，可用 `--reduce-motion` 作诊断对照：

```powershell
cargo run --locked --release -p gpui-material-gallery -- button
cargo run --locked --release -p gpui-material-gallery --features frame-profile -- button --frame-profile
cargo test --locked --release -p gpui-material-foundation --lib ripple_clip_benchmark -- --ignored --nocapture
cargo test --locked --release -p gpui-material-gallery --features test-support --lib perf_bench -- --ignored --nocapture --test-threads=1
```

- `perf_bench` 在固定视口下按三层拆解帧开销：**引擎层**（`engine/*`）、独立**组件层**（`component/*`）以及**示例页与 Gallery 外壳层**（`story/*` 与 `gallery/*`）。
- `ripple_clip_benchmark` 单独测量预构建涟漪网格的裁剪耗时。
- `--frame-profile` 对单个原生窗口预热 2 秒后采样 10 秒，输出 CPU 绘制耗时与平台帧提交间隔的 p50/p95/p99。

如需测量冷/热构建耗时，将报告输出到新的目录：

```powershell
cargo fetch --locked --target x86_64-pc-windows-msvc
cargo xtask timings target/timings/gallery-timings
```

<a id="documentation-website"></a>

## 文档网站

[网站源码](../website/)使用 Astro、[MDUI](https://www.mdui.org/zh-cn/) Web Components 与 Pagefind 搜索。安装 Node.js 24 和 npm 后运行：

```sh
npm --prefix website ci
npm --prefix website run dev
npm --prefix website run build
npm --prefix website run preview
```

开发服务器使用 `/`；生产构建默认使用 `/gpui-material/`，输出到 `website/dist/` 并包含 Pagefind 索引。

共用指南在 `docs/` 编辑，控件示例在 `website/component/` 编辑（同一控件页面的中英文共用一份 Rust 示例）。网站组件目录和侧栏分为基础、表单、布局、高级四组，各组内按 Rust 名称排序。

浏览器预览将 Rust Gallery 编译为 WebAssembly，每个演示在独立 GPUI 画布中运行（需支持 WebGPU 或 WebGL2）。在本地启用预览时，启动 Astro 前先构建 WASM 资源：

```sh
rustup toolchain install nightly-2026-10-31 --profile minimal --target wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
npm --prefix website run build:wasm
```

在仓库设置中将 **Settings → Pages → Build and deployment → Source** 设为 **GitHub Actions**。修改网站源码或 WASM 构建输入的 PR 会运行网站构建；`main` 上更新网站、构建输入或 `docs/` 时会自动部署。

<a id="ci-and-merging"></a>

## 持续集成与合并

[Engineering CI](../.github/workflows/ci.yml) 通过对比检出内容与基准提交选择检查作业（手动运行会执行全部作业）。

| 作业 | 检查内容 |
| --- | --- |
| `Change scope` | 判断变更范围并检查差异空白。 |
| `Documentation checks` | 文档和工程变更运行 `cargo xtask docs --check`。 |
| `Website / Build documentation` | 网站源码或构建输入变更时构建带 WASM Gallery 的网站并检查链接。 |
| `Repository checks` | 检查 Rust 格式、Token 生成物、三平台依赖图及默认工作区。 |
| `Windows MSVC`、`macOS ARM64`、`Linux x64` | 运行 Clippy、工作区测试、Gallery 无头 UI 测试、原生 Gallery 构建与 Rustdoc 检查（Linux x64）。 |
| `CI required` | 核验选中的作业全部通过、未选中的作业均已跳过。 |

Rust 构建缓存按平台隔离，由 `main` 保存并供 PR 复用。更新 `main` 须通过基于最新主线且 `CI required` 通过的 PR，详见[评审与合并规则](../CONTRIBUTING.zh.md#pr-评审与合并)。发版流程见[版本与发布](release.zh.md#prepare-and-publish)。
