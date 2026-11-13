# Phase progression and repairs

[简体中文](phases.md) · English

[status.toml](../status.toml) is the machine-readable source of phase status. Phases proceed in order from P0 through P9. A phase starts implementation after its predecessor passes acceptance and freezes. Only one phase can be `active` or in `review`. `planned` indicates a future phase, `blocked` identifies a named blocker, and `frozen` records evidence and an acceptance commit.

| Phase | Scope | Exit criteria |
| --- | --- | --- |
| [P0](phases/P00.en.md) | Requirements, official sources, coverage matrix, platforms, dependency contracts | Required scope and classifications are clear. Each item has a source, owner, and acceptance direction. P1 has runnable probe tasks. |
| [P1](phases/P01.en.md) | Isolated probes, architecture, and dependency direction | Required capabilities have passing evidence. Adapters or patches needed to unblock them are verified. |
| [P2](phases/P02.en.md) | Workspace, toolchain, CI/CD, minimal test window, Gallery host, xtask, templates, and merge checks | Windows builds and runs reproducibly. Initial release verification and measurement are executable. |
| [P3](phases/P03.en.md) | Theme crate, tokens, color, typography, shape, MotionSpec, layout, and density | Pure crate graphs exclude GPUI. Token parsing, generation, and comparisons with official algorithms pass. |
| P4 | Foundation and drawing, input, focus, overlay, drag, scroll, and semantic primitives | Each primitive is verified independently, with platform and adapter evidence. |
| P5 | Basic controls, display feedback, first facade, and Gallery scenarios | Applicable variants and behaviors pass component checks. The 0.0.1 preview can ship. |
| P6 | Text fields and overlays | Chinese IME, nested overlays, focus restoration, and actual assistive technology pass. |
| P7 | Navigation and adaptive layout | State survives resizing and mode changes. Keyboard, scroll, and RTL checks pass. The 0.0.2 preview can ship. |
| P8 | Composite and extended components | All required items have verified evidence. Experiments have a recorded decision. The 0.0.3 candidate can ship. |
| P9 | Integration, support matrix, runtime and build regression, and release | Support claims, dependencies and MSRV, licensing, API stability, and migration guidance are complete. The release is reproducible and 0.1.0 can ship. |

P1 probes live in `experiments/`. Production modules enter the repository in their first implementation phase. The owning phase delivers a component's behavior, variants, documentation, and acceptance. A Cargo package can contain frozen and current-phase functionality.

## GitHub workflow

Issues record independently verifiable deliverables, prerequisites, file scope, pass criteria, and evidence. PRs link the issue and describe the change and its verification. The phase freeze collects merged deliverables, evidence, and the acceptance commit. Review both functionality and phase boundaries. [Development and verification](development.en.md)

Assign one owner to shared contracts. Split independent work by issue and file ownership, use isolated workspaces, and integrate in dependency order. Follow upstream contribution rules for verified general defects, with a minimal reproduction and patch. Track review, merge, release, and local migration evidence in the project issue.

## Maintenance after a freeze

Between phases, use a dedicated issue and PR for Markdown workflows and documentation links. When moving evidence, map the original path from the freeze commit to its current location in `status.toml` under `evidence_locations`. Keep acceptance commits, original evidence paths, and delivery records traceable. Include related status and validation changes in the same PR. Use the repair process when changing a frozen contract or behavior.

GitHub maintenance can update `.github/workflows/ci.yml`, the PR template, and Markdown templates under `.github/ISSUE_TEMPLATE/`. The same PR can update the corresponding phase path check. Frozen records must still pass validation.

Maintenance of the generated token index can update its generator in the same PR. Change token data and generated Rust tables in the owning phase or through the repair process.

A maintenance PR can retire one-off verification from completed isolated experiments. Keep historical results and commits in the phase evidence. The path gate permits maintenance of P1 experiments and related gate changes in the same PR.

## Repairs

1. Block the current phase and record the failure evidence.
2. In the status file, record the owning phase, scope, reproduction, and evidence. Reopen only the phase that owns the defect.
3. Fix the defect and run verification at that layer and regressions for affected downstream behavior.
4. Update the original topic document and acceptance evidence, record the new freeze commit, and resume the previous phase.

Phase acceptance checks scope, conformance between specification and implementation, applicable states and platforms, real user entry points, licensing, and dependency changes. Examples and tests exercise runnable production implementations. Record design decisions in the corresponding topic documents and use Git for traceability.
