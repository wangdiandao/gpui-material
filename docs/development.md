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

每次传入新的报告目录。Cargo 的注册表和源码缓存已预取，冷检查与冷构建各使用自己的空目标目录。随后用同一命令与同一目录测热结果。工具保存四次 wall time、HTML 时间线、Git 提交、目标和工具链。时间线中的 Gallery 最后一个编译单元包含代码生成和链接，可用于定位构建成本。测量使用普通 Gallery feature、开发 profile、无并行 Cargo 作业。比较结果时记录 OS、CPU、运行环境与缓存策略。验收后归档 `summary.md` 和四份 HTML，再清理报告目录里的 `check-target/` 与 `build-target/`。

## 持续集成与合并

[Engineering CI](../.github/workflows/ci.yml) 在面向 `main` 的 PR、`main` 更新和手动触发时运行。托管的 `Windows MSVC` 作业执行格式、阶段、文档、依赖、Clippy、测试和 Gallery 构建。GitHub Actions 的 `gallery-windows-x64-msvc` 制品包含可执行文件、Cargo 时间线及许可记录，供 Windows 实机复验。正式发布由对应交付阶段启动。

Windows 下载制品使用 `cargo build --locked -p gpui-material-gallery --profile gallery-artifact --timings`。此 profile 关闭 debug assertions，GPUI 通过 Windows SDK 的 x64 `fxc.exe` 在构建时嵌入 DirectX 着色器。CI 通过 `GPUI_FXC_PATH` 指定 SDK 中的编译器。常规 `cargo gallery` 在开发机上从 Cargo 缓存读取着色器。

提交 PR 时关联阶段 Issue，按模板记录环境、命令和结果。GitHub 要求通过 PR 更新 `main`、基于最新主线、解决评审对话，并通过 GitHub Actions 的 `Windows MSVC` 检查。评审核对文件归属、阶段范围、验证证据和集成顺序。

## PR 合并后收尾

每个任务结束时由交付人收尾。日常开发在主检出中切换分支即可。并行工作使用按 Issue 编号命名的 worktree，集中放在本机约定的工作区目录，例如 `E:/repos/worktrees/gpui-material/66`。源码 worktree 放在 Cargo 目标目录之外。

1. **归档证据。** 将验收结论与必要截图提交到对应阶段文档。较大的日志、报告和可执行文件使用有明确保留期限的制品，长期需要的原始记录统一存放在 `%LOCALAPPDATA%/gpui-material/evidence/` 的阶段或 Issue 子目录。对照验收记录核对已归档内容，再清理本机生成物。
2. **同步主线。** 在主检出运行 `git fetch --prune origin`、`git switch main` 和 `git pull --ff-only origin main`。对待清理分支运行 `git merge-base --is-ancestor <branch> origin/main`，退出码 0 表示该分支的提交都在主线历史中。Squash 或 rebase 合并后，先核对 PR 的合并记录、合并时的 head 和本地分支有无新增提交。存在未交付提交、打开的 PR 或仍被下游使用时保留分支。
3. **移除 worktree。** 用 `git worktree list` 确认路径，用 `git -C <worktree> status --short --untracked-files=all` 和 `git -C <worktree> ls-files --others --ignored --exclude-standard --directory` 检查未提交与忽略文件。归档证据并清理确认可重建的缓存后，从主检出执行 `git worktree remove <worktree>`。遇到脏工作区先查明内容。手动丢失目录后用 `git worktree prune --dry-run` 检查登记，再用 `git worktree prune` 清除失效登记。
4. **删除已交付分支。** worktree 移除后，在 `main` 上执行 `git branch -d <branch>`。常规短期 PR 分支可在 GitHub 删除远端分支，仓库也可启用合并后自动删除。随后 `git fetch --prune origin` 清掉本地的远端跟踪引用。受保护分支、发布分支与仍有用途的验证分支按各自生命周期处理。

### Cargo 目标目录

日常保留主检出的 `target/`，用于下一次增量构建。临时 worktree 使用独立默认缓存，或将该任务 shell 的 `CARGO_TARGET_DIR` 指向主检出下独立的 `target/worktrees/<issue>`。目录归属随任务记录，清理时先关闭相关程序和 Cargo 进程。共享缓存由主检出统一清理，单个 PR 合并时继续复用。

需要释放空间时，先核对目标目录，预览并清理指定缓存。根 Workspace 的清理只针对自己的目标目录，P1 探针使用独立清单和缓存。以下命令在主检出根目录运行。

```powershell
cargo clean --dry-run --target-dir target
cargo clean --target-dir target
cargo clean --dry-run --manifest-path experiments/p1-host/Cargo.toml --target-dir experiments/p1-host/target
cargo clean --manifest-path experiments/p1-host/Cargo.toml --target-dir experiments/p1-host/target
```

同样可为 `p1-theme`、`p1-shapes` 或指定的 worktree 缓存使用对应清单和目标路径。全量 `cargo clean` 会一起删除目标目录内的自定义日志、截图和报告，先完成归档。Rust 工具链及全局 Cargo 下载缓存可供后续任务复用。
