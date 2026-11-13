# 平台与质量要求

Windows 是主要开发和验收平台。P1 起为 macOS 和 Linux 运行同一必需能力探针。下表是验证环境，实际 OS、架构、后端、工具链、设备和工具版本写入每次证据。各平台当前均为待验证。

| 平台 | 验证环境 | 真实窗口与系统检查 |
| --- | --- | --- |
| Windows | Windows 11 x64，MSVC，物理机或有桌面会话的运行器；主机原生 DPI 与跨显示器缩放 | GPUI 真实窗口截图；UI Automation 树和 Narrator 实际操作；微软拼音组合与候选定位；滚轮、触控板与可用触摸设备。 |
| macOS | Apple Silicon 上的 macOS，Xcode/Metal，签名应用包，真实 GUI 会话 | Metal 无头像素和真实窗口；Accessibility Inspector 与 VoiceOver；系统拼音、Retina、全键盘访问。Intel 支持由独立验证决定。 |
| Linux | x86_64 Ubuntu 24.04 LTS / GNOME，Wayland 与 X11 各走一轮；Vulkan 硬件或 CI 软件适配器 | WGPU 无头像素；Wayland/X11 实窗，AT-SPI 树与 Orca，IBus 中文输入、分数缩放；记录发行版与会话。 |

上游固定版本的测试 harness 支持三平台的交互与布局。macOS 和 Linux 采用对应无头渲染器。Windows 视觉证据来自真实窗口。缺失像素或实机证据记为未验证。依据 [GPUI Kit 固定测试说明][kit-test]与[无障碍说明][kit-a11y]。

## 各层证据

| 层次 | 覆盖重点 |
| --- | --- |
| 纯逻辑 | Token 解析、算法向量、形状与动画参数、边界值、状态转移。 |
| 组件交互 | 生产组件渲染后的指针、键盘、语义动作，稳定身份、焦点、拖动取消、禁用、状态所有权和回调。 |
| 视觉 | 浅深主题、字体、可用宽度、DPI、动画采样点和布局边界。固定环境比对；记录视觉误差及接受理由。 |
| 原生系统 | 中文 IME 预编辑、候选位置和提交、剪贴板、失焦、系统低动效、缩放和跨显示器、真实读屏。 |
| 集成 | 触发菜单、对话框内输入、导航模式切换、搜索结果回写、日期/时间编辑等完整路径。 |
| 性能 | 活动帧耗时和分配、长列表、退出/静止重绘、主题更新、构建时间。P1/P2 记录基线，阶段实现中保持可复测。 |

通用样本覆盖中文及长标签、LTR/RTL、浅/深色、正常/缩放字体、窗口变窄及高 DPI。高风险组合包括 disabled 与 hover/pressed、selected 与 focus、滚动中的浮层与 IME、动画中断、过期异步结果。辅助技术用角色、名称、值、状态、关系和动作验证语义，实际读屏验证播报和操作，视觉证据验证可见焦点。

每份证据记录提交、组件与变体、输入路径、测试或复现步骤、运行环境、期望与观察结果。状态可用 pending、pass、fail、unsupported。所声明范围内的必需项需有 pass 证据。平台声明要明确 OS、架构、后端、输入法/辅助技术与例外范围，并通过对应必需组件验收、原生探针与集成回归。

[kit-test]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/website/docs/test.md
[kit-a11y]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/website/docs/accessibility.md
