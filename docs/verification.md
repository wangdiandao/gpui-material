# 平台与质量要求

简体中文 · [English](verification.en.md)

首版支持 Windows 11 x64（MSVC），所有必需能力和发布验收在此平台完成。macOS 和 Linux 的探针结果作为探索记录。这两个平台的支持声明需经各自的完整实机与组件验收。

| 平台 | 首版范围 | 验证环境和真实系统检查 |
| --- | --- | --- |
| Windows | 必需 | Windows 11 x64，MSVC，物理机或带桌面会话的运行器。真实窗口、原生 DPI 与跨显示器缩放、UI Automation、Narrator、微软拼音组合与候选定位、滚轮、触控板及可用触摸设备。 |
| macOS | 探索 | Apple Silicon、Xcode/Metal、签名应用包。可记录 Metal 无头像素、实窗、Accessibility Inspector、VoiceOver、系统拼音、Retina 与全键盘访问。 |
| Linux | 探索 | x86_64 Ubuntu 24.04 LTS / GNOME。可记录 WGPU 像素、Wayland/X11 实窗、AT-SPI、Orca、IBus 中文输入与分数缩放。 |

每次记录 OS、架构、后端、工具链、设备与工具版本。Windows 的视觉证据来自真实窗口，macOS 和 Linux 可用对应无头渲染器采集像素证据。缺失像素或实机证据记为未验证。依据 [GPUI Kit 固定测试说明][kit-test]与[无障碍说明][kit-a11y]。

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
