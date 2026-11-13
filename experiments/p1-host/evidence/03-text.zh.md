# P1-03 输入与文本度量

[English](03-text.md) | 中文

2026-11-13，Windows 11 x64 build 26200，Rust/Cargo 1.99.1，使用此 PR 的代码与已锁定的 crates.io 依赖。实窗缩放 100%，UIA 和截图工具为 winapp 0.7.1。

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin text
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin text
```

| 场景 | 观察 | 状态 |
| --- | --- | --- |
| Base InputHandler 预编辑、确认与取消 | 先写入一个代理对字符，预编辑标记为 UTF-16 2..7，确认后值为 `🙂中`；第二次预编辑取消后值保持 `🙂中`。| pass |
| Windows 原生密码字段 | 使用非空固定样本 `P1-sample`。[截图](03-windows.png)显示掩码，`winapp ui get-property probe-password -p IsPassword` 返回 True，`get-value` 返回可访问名称“密码”。 | pass |
| Windows 中文 IME | Microsoft Pinyin 已安装。winapp 的点击与 SendInput 都因前台焦点检查而退出。 | pending |
| macOS 与 Linux 中文 IME、各平台候选定位、失焦、浮层、剪贴板及字形度量 | 待在对应会话和真实控件上运行。 | pending |

<a id="additional-editing-regressions"></a>

## 补充编辑回归

2026-11-13，Windows 11 x64，Rust 1.99.1。用同一测试命令验证三项。

- 通过 Base 输入框的选择、复制、删除和粘贴快捷键，`中文🙂` 从探针剪贴板往返。对预填非空值的密码框执行复制时，剪贴板不含密码样本。
- 通过指针打开 Popover，浮层内的 InputHandler 预编辑并提交「中」。候选锚点的处理器矩形在输入框起点之后，Esc 关闭后焦点回到触发按钮并保留输入。
- 实机可用「打开浮层输入…」复测原生候选框、确认、取消、滚动或缩放后的定位。无头测试的剪贴板和候选锚点属于 GPUI 测试平台。

<a id="windows-directwrite-fallback-and-metrics"></a>

## Windows DirectWrite 字体回退与度量

2026-11-13，Windows 11 x64 build 26200，Rust 1.99.1，agent-desktop 1.2.0。以 `P1_TEXT_METRICS=1` 启动同一文本探针。Canvas prepaint 用 Consolas 24 px 排 `A中文B`，同一 `ShapedLine` 用于画字、字符位置量测和行尾标记。

- 拉丁段使用 FontId(1)，中文段使用回退 FontId(0)。两字起点为 13.1953125 px 和 37.195313 px，整行 advance 为 74.390625 px。
- GPUI 断言末尾索引与 advance 相同。100% 缩放的 [Windows 截图](03-windows-metrics.png) 可见完整中英文和绘制在末尾的标记。
- 结合输入处理器回归与 [Windows 实机确认](../../../docs/phases/P01-evidence.zh.md#native-windows-acceptance)，记录 P1-03 通过。
