# Contributing to gpui-material

English | [中文](CONTRIBUTION.zh.md)

## Issues and pull requests

- Check the [documentation index](docs/index.md) and [phase status](status.toml) for the owning phase and its scope. Search existing issues and pull requests before opening a new one.
- Use the repository's issue templates. For bugs, include a minimal reproduction, environment, and expected and observed behavior. For proposals, include the use case and acceptance criteria.
- Branch from the latest `main` and name the branch `<type>/<short-topic>`. Keep each PR focused on one problem and link the issue it addresses. Describe any public API changes and migration steps. [Branch naming conventions](docs/development.md#contribution-workflow-and-branch-naming)
- Run the relevant checks in the [development guide](docs/development.md#testing-conventions). Record reproducible commands and results in the PR. For UI changes, include the affected Gallery scenario and before-and-after evidence.

## Issues and contributions upstream

File issues and contribute fixes to the upstream project that owns behavior outside gpui-material's scope. Material themes, drawing, and presentation belong here. Reusable control behavior and accessibility semantics belong in GPUI Kit's `gpui-base`. GPUI windowing, rendering, and platform input belong in Zed's GPUI. [Project architecture](docs/architecture.md)

1. Reduce the problem to a minimal reproduction on the affected upstream version. Check the current upstream branch and search for an existing issue or PR.
2. Follow that repository's contribution, testing, licensing, and review rules. Keep the patch to the smallest scope and size that fixes the problem and verifies the affected behavior. [GPUI Kit contribution guide](https://github.com/longbridge/gpui-kit/blob/main/CONTRIBUTING.md) · [Zed contribution guide](https://github.com/zed-industries/zed/blob/main/CONTRIBUTING.md)
3. If the problem affects gpui-material, link the upstream issue or PR from a local tracking issue. Record the fixed release, dependency update, regression checks, and conditions for removing any temporary adapter. [Upstream contributions](docs/development.md#upstream-contributions)
