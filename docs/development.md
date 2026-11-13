# 开发与验证

在 Windows 11 x64 安装 [Rustup](https://rustup.rs/) 和 Visual Studio Build Tools 的 C++ 桌面开发工作负载，包含 MSVC x64 和 Windows SDK。根目录的工具链文件固定 Rust、rustfmt 与 Clippy。初次构建需下载锁文件所列包。

```powershell
rustc -vV
cargo check --locked
cargo gallery
cargo test --locked -p gpui-material-gallery --features test-support
```

默认 Workspace 构建运行纯工程工具。Gallery 用 `-p gpui-material-gallery` 或 `cargo gallery` 显式选择。正常 Gallery 构建采用平台默认行为，UI 集成测试启用 `test-support`。P1 的独立探针可以继续通过各自的 `--manifest-path` 运行。

Gallery 标题栏显示「Material Gallery」。确认正文显示「Window check」及与当前显示器一致的「Display scale」。缩窄窗口时确认文字仍可见。Windows 的 UI Automation 树可检查名为「Display scale …%」的 StatusBar。使用标题栏关闭按钮或 Alt+F4，进程应正常退出。记录 Windows 版本、目标、缩放、截图、语义和退出结果。组件交互与更广的原生验收见[平台与质量要求](verification.md)。

## 本地检查

```powershell
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo xtask check
cargo xtask check origin/main
```

`check` 校验阶段状态、冻结证据与接受提交祖先关系、已跟踪 Markdown 的仓库内链接和标题锚点，以及 GPUI/Base 的包身份与 feature。传入基线提交后，使用 Git 提交比较路径并依据基线的阶段范围校验。目录条目以斜杠结束，文件条目按完整路径匹配。冻结所需证据先合入接受提交，再以单独 PR 登记冻结。文档检查无需访问外部链接。依赖检查解析 Windows、macOS 和 Linux 的普通及测试图，托管编译及首版支持验收以 Windows 为准。

## 冷热构建基线

```powershell
cargo fetch --locked --target x86_64-pc-windows-msvc
cargo xtask timings target/timings/p2-baseline
```

每次传入新的报告目录。Cargo 的注册表和源码缓存已预取，冷检查与冷构建各使用自己的空目标目录。随后用同一命令与同一目录测热结果。工具保存四次 wall time、HTML 时间线、Git 提交、目标和工具链。时间线中的 Gallery 最后一个编译单元包含代码生成和链接，可用于定位构建成本。测量使用普通 Gallery feature、开发 profile、无并行 Cargo 作业。比较结果时记录 OS、CPU、运行环境与缓存策略。报告保留两个 Cargo 目标目录供复查，体积随依赖增长。

## 持续集成与合并

[Engineering CI](../.github/workflows/ci.yml) 在面向 `main` 的 PR、`main` 更新和手动触发时运行。托管的 `Windows MSVC` 作业执行格式、阶段、文档、依赖、Clippy、测试和普通 Gallery 构建。GitHub Actions 的 `gallery-windows-x64-msvc` 制品包含可执行文件、Cargo 时间线及许可记录，供 Windows 实机复验。正式发布由对应交付阶段启动。

提交 PR 时关联阶段 Issue，按模板记录环境、命令和结果。GitHub 要求 PR 基于最新 `main`，且 GitHub Actions 的 `Windows MSVC` 检查成功。评审核对文件归属、阶段范围、验证证据和集成顺序。
