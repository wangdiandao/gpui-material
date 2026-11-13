# Platforms and quality requirements

[简体中文](verification.md) · English

The initial release supports Windows 11 x64 (MSVC). Required capabilities and release acceptance are verified on that platform. macOS and Linux probe results are exploratory. A support declaration for either platform requires full native and component acceptance.

| Platform | Initial release | Environment and native checks |
| --- | --- | --- |
| Windows | Required | Windows 11 x64 with MSVC on a physical machine or a runner with a desktop session. Check real windows, native DPI and cross-display scaling, UI Automation, Narrator, Microsoft Pinyin composition and candidate positioning, mouse wheel, touchpad, and available touch hardware. |
| macOS | Exploratory | Apple Silicon, Xcode/Metal, and a signed application bundle. Record Metal headless pixels, a real window, Accessibility Inspector, VoiceOver, system Pinyin, Retina, and full keyboard access where available. |
| Linux | Exploratory | x86_64 Ubuntu 24.04 LTS with GNOME. Record WGPU pixels, real Wayland and X11 windows, AT-SPI, Orca, IBus Chinese input, and fractional scaling where available. |

Record the OS, architecture, backend, toolchain, device, and tool versions for each run. Capture Windows visual evidence from a real window. macOS and Linux may use their headless renderers for pixel evidence. Record missing pixel or native evidence as unverified. [Pinned GPUI Kit testing guide][kit-test] · [Accessibility guide][kit-a11y]

## Evidence by layer

| Layer | Coverage |
| --- | --- |
| Pure logic | Token parsing, algorithm vectors, shape and animation parameters, boundary values, and state transitions. |
| Component interaction | Pointer, keyboard, and semantic actions on rendered production components, plus stable identity, focus, drag cancellation, disabled state, state ownership, and callbacks. |
| Visual | Light and dark themes, fonts, available width, DPI, animation sample points, and layout boundaries. Compare in a fixed environment and record visual deviations and acceptance rationale. |
| Native system | Chinese IME preedit, candidate positioning and commit, clipboard, focus loss, system reduced motion, scaling and display transitions, and actual screen readers. |
| Integration | Complete paths such as menu activation, input inside dialogs, navigation mode changes, applying search results, and date and time editing. |
| Performance | Active frame time and allocation, long lists, exit and idle redraw, theme updates, and build time. P1 and P2 establish repeatable baselines for later phases. |

Shared scenarios cover Chinese and long labels, LTR and RTL, light and dark themes, normal and scaled fonts, narrow windows, and high DPI. High-risk combinations include disabled with hover or press, selected with focus, overlays and IME during scrolling, interrupted animations, and stale async results. Verify semantic roles, names, values, states, relationships, and actions with accessibility tools. Use actual screen readers to verify announcements and interaction, and visual evidence for visible focus.

Each evidence record names the commit, component and variant, input path, test or reproduction steps, environment, expected result, and observed result. Use `pending`, `pass`, `fail`, or `unsupported`. Each required item in a support claim needs passing evidence. The claim specifies the OS, architecture, backend, IME and assistive technology, and exceptions, and must pass the applicable component, native, and integration checks.

[kit-test]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/website/docs/test.md
[kit-a11y]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/website/docs/accessibility.md
