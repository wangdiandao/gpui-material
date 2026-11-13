# P1-03 输入与文本度量

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
