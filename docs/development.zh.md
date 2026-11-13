# 开发与验证

[English](development.md) | 中文

<a id="contribution-workflow-and-branch-naming"></a>

## 贡献流程与分支命名

- 从 `.github/ISSUE_TEMPLATE/` 选择缺陷报告、功能提议或阶段交付。缺陷附最小复现，提议交代场景与验收，阶段 Issue 记录前置依赖和文件归属。
- 从最新 `main` 建分支，使用 `<类型>/<Issue 编号>-<简短主题>`，主题使用小写英文和连字符。无关联 Issue 时使用 `<类型>/<简短主题>`，如 `ci/cross-platform-workflow`。
- 按改动目的选前缀。`feat/` 新功能，`fix/` 缺陷，`docs/` 文档，`test/` 测试，`refactor/` 重构，`ci/` CI，`chore/` 工程维护，`release/` 发布。例如 `fix/48-pointer-capture`。阶段归属由 Issue、PR 和 `status.toml` 记录。
- 每个 PR 解决一个问题。按模板关联 Issue，列出公开 API 及迁移方式，记录可复现的验证。界面改动提供 Gallery 入口和前后截图或录屏。[GPUI Kit 贡献指南](https://github.com/longbridge/gpui-kit/blob/main/CONTRIBUTING.md)

<a id="workspace-and-component-development"></a>

## Workspace 与组件开发

- 生产库按[架构与依赖](architecture.zh.md)放在 `crates/`，组件按族组织。Gallery 位于 `apps/gallery/`，调用生产公开 API，随组件交付可运行的演示。参照 GPUI Component 的组件库与 Story 分工。[组件库](https://github.com/longbridge/gpui-kit/tree/39ab849a93b5f15507b969164d49891db8d5b470/crates/component) · [Story](https://github.com/longbridge/gpui-kit/tree/39ab849a93b5f15507b969164d49891db8d5b470/crates/story)
- 组件改动在所属 crate 验证公共行为，并从 Gallery 复测相关变体和交互。可独立复现的场景使用最小入口。`experiments/` 存放隔离探针，`xtask/` 存放工程检查和离线生成。

<a id="coding-conventions"></a>

## 编码规范

- 从[文档索引](index.zh.md)定位当前规格，再查[阶段状态](../status.toml)的允许路径。已冻结契约的修订遵循[返修流程](phases.zh.md#repairs)。
- 英文 `foo.md`、中文 `foo.zh.md` 与逐节指纹 `foo.i18n.yaml` 放在同一目录。标题下链接另一语言，正文链接优先指向同语言页面。在同一 PR 同步两页，确认内容对应后运行 `cargo xtask docs --write path/to/foo.md` 更新记录。`cargo xtask check` 核验配对、语言入口、标题层级、代码示例和指纹。GitHub Issue 和 PR 模板使用英文。[DeepSeek Harness 文档约定](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/i18n/README.md)
- theme 保持纯 Rust，foundation 负责 GPUI/Base 适配。应用或 Base 持有业务与输入状态，Material 持有呈现和新增组合状态。优先复用现有公共接口。[架构与依赖](architecture.zh.md)
- 采用 Rust 惯用命名和稳定的语义标识。注释说明不变量、生命周期与平台差异。公开文档交代状态所有权、默认值、焦点、回调和平台支持。

<a id="testing-conventions"></a>

## 测试规范

- 每次变更都验证受影响的行为。先用已有测试、构建检查或最小复现。阶段规格要求、公共行为或关键不变量缺少覆盖，或可稳定复现的缺陷仍有实际复发风险时，新增长期自动化测试。仅改文档、格式或内部结构且已有覆盖足够时，运行相关检查即可。
- 在最低有效层验证可观察结果。同一行为优先补到已有用例，只有上层存在独有风险才增加 UI、集成或原生验证。用例应可重复运行，并能说明覆盖的契约和退化时的失败表现。组件按适用场景验证真实渲染、键盘、指针、语义、焦点、取消和原生系统路径。[平台与质量要求](verification.zh.md)
- 单次修复验证、环境排查、迁移和阶段探测可用临时命令或脚本。重要结果和复现步骤写入对应文档。只有覆盖仍受支持且现有测试漏掉的回归时，才将最小断言纳入长期套件。
- 修改行为或替换探针时，清理已被正式测试覆盖、失去验证对象或仅用于一次排查的用例及辅助代码。删除前核对独有覆盖与引用。

<a id="upstream-contributions"></a>

## 上游贡献

- 确认通用上游缺陷时，先写最小复现并检查现有 Issue 和 PR。遵守上游贡献、测试、许可与评审规则。
- 补丁聚焦通用缺口并附最小验证。用项目 Issue 跟踪上游评审、合并、发布和临时适配的退出条件。
- 更新固定依赖前验证包身份、平台回归和影响范围。以对应上游发布记录核验版本、提交与源码。

<a id="development-environment"></a>

## 开发环境

在 Windows 11 x64 安装 [Rustup](https://rustup.rs/) 和 Visual Studio Build Tools 的 C++ 桌面开发工作负载，包含 MSVC x64 和 Windows SDK。根目录的工具链文件固定 Rust、rustfmt 与 Clippy。初次构建需下载锁文件所列包。

```powershell
rustc -vV
cargo check --locked
cargo gallery
cargo test --locked -p gpui-material-gallery --features test-support
```

默认 Workspace 构建包含纯主题和工程工具。Gallery 用 `-p gpui-material-gallery` 或 `cargo gallery` 显式选择。正常 Gallery 构建采用平台默认行为，UI 集成测试启用 `test-support`。P1 的独立探针可以继续通过各自的 `--manifest-path` 运行。

macOS 使用 Apple Silicon 和 Xcode 命令行工具。Linux CI 使用 Ubuntu 24.04 x64，系统依赖见[工作流](../.github/workflows/ci.yml)中的 `Install Linux platform dependencies` 步骤。两平台使用同一 Rust 工具链与 Cargo 命令。

Gallery 标题栏显示「Material Gallery」。确认正文显示「Window check」及与当前显示器一致的「Display scale」。缩窄窗口时确认文字仍可见。Windows 的 UI Automation 树可检查名为「Display scale …%」的 StatusBar。使用标题栏关闭按钮或 Alt+F4，进程应正常退出。记录 Windows 版本、目标、缩放、截图、语义和退出结果。组件交互与更广的原生验收见[平台与质量要求](verification.zh.md)。

<a id="local-checks"></a>

## 本地检查

```powershell
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo xtask check
cargo xtask check origin/main
```

`check` 校验阶段状态、冻结证据与接受提交祖先关系、已跟踪 Markdown 的仓库内链接和标题锚点，以及 GPUI/Base 的包身份与 feature。传入基线提交后，使用 Git 提交比较路径并依据基线的阶段范围校验。目录条目以斜杠结束，文件条目按完整路径匹配。冻结所需证据先合入接受提交，再以单独 PR 登记冻结。文档检查无需访问外部链接。Token 生成物检查可单独运行 `cargo xtask tokens --check`。修改数据后用 `cargo xtask tokens` 更新 Rust 表与索引。见[纯主题](theme/theme.zh.md#token-data)。依赖检查解析 Windows、macOS 和 Linux 的普通及测试图。

<a id="cold-and-warm-build-baseline"></a>

## 冷热构建基线

```powershell
cargo fetch --locked --target x86_64-pc-windows-msvc
cargo xtask timings target/timings/p2-baseline
```

每次传入新的报告目录。Cargo 的注册表和源码缓存已预取，冷检查与冷构建各使用自己的空目标目录。随后用同一命令与同一目录测热结果。工具保存四次 wall time、HTML 时间线、Git 提交、目标和工具链。时间线中的 Gallery 最后一个编译单元包含代码生成和链接，可用于定位构建成本。测量使用普通 Gallery feature、开发 profile、无并行 Cargo 作业。比较结果时记录 OS、CPU、运行环境与缓存策略。验收记录写明关键计时结果及复现命令。

<a id="ci-and-merging"></a>

## 持续集成与合并

[Engineering CI](../.github/workflows/ci.yml) 在面向 `main` 的 PR、`main` 更新和手动触发时运行。作业划分参照 [GPUI Kit CI](https://github.com/longbridge/gpui-kit/blob/main/.github/workflows/ci.yml)。

| 作业 | 检查 |
| --- | --- |
| `Repository checks` | 格式、差异空白、阶段范围、文档链接、Token 生成物、三平台依赖图及默认 Workspace。PR 和主线 push 使用对应基线提交，手动运行执行无基线检查。 |
| `Windows MSVC`、`macOS ARM64`、`Linux x64` | 在 Windows Server 2025 x64、macOS 15 ARM64 和 Ubuntu 24.04 x64 上分别运行 Clippy、工作区测试、Gallery 无头 UI 测试和原生 Gallery 构建。平台失败后其余平台继续执行。 |
| `CI required` | 通用检查与三个原生作业全部通过时成功。 |

Rust 构建缓存按平台隔离，由 `main` 的成功 push 更新，PR 可复用。P1 探针工作流已在 GitHub 停用，并从验证分支删除。历史运行与原生探针证据见 [P1 验证记录](phases/P01-evidence.zh.md)。[删除提交](https://github.com/wangdiandao/gpui-material/commit/c0596fede13cd4663f28b1eff8bc72ff2f1d5014)

三平台托管验证覆盖构建与自动化测试，首版原生实机支持按[平台与质量要求](verification.zh.md)验收。

Windows 作业上传 `gallery-windows-x64-msvc`，包含可执行文件、Cargo 时间线及许可记录，供实机复验。制品使用 `cargo build --locked -p gpui-material-gallery --profile gallery-artifact --timings`。此 profile 关闭 debug assertions，GPUI 通过 Windows SDK 的 x64 `fxc.exe` 在构建时嵌入 DirectX 着色器。CI 通过 `GPUI_FXC_PATH` 指定 SDK 中的编译器。常规 `cargo gallery` 在开发机上从 Cargo 缓存读取着色器。正式发布由对应交付阶段启动。

GitHub 规则集要求通过 PR 更新 `main`、基于最新主线并解决评审对话。迁移期间保留现有的 `Windows MSVC` 必检项。新工作流合入主线且 `CI required` 首次成功后，将必检项切换为 GitHub Actions 的 `CI required`。

<a id="first-cross-platform-ci-run"></a>

### 跨平台 CI 首次验证

2026-11-13 在 `ci/cross-platform-workflow` 手动验证 `1a28571be2058ca237aaf04a0552f2e1d69aa098`。通用检查、Linux x64、macOS ARM64、Windows MSVC 和 `CI required` 全部通过。三个原生作业分别耗时 3 分 45 秒、4 分 37 秒、7 分 17 秒。复现命令为 `gh workflow run ci.yml --ref ci/cross-platform-workflow`。PR 基线的路径门禁在本地用 `cargo xtask check origin/main` 验证通过。[Actions 运行](https://github.com/wangdiandao/gpui-material/actions/runs/43416482503) · [Windows Gallery 制品](https://github.com/wangdiandao/gpui-material/actions/runs/43416482503/artifacts/14032242512)
