# Attribution / 来源与署名

## Shuffle

Shuffle Flow is derived from [WizenPainter/shuffle](https://github.com/WizenPainter/shuffle), created by **Jaime Guzman**, under the [MIT license](LICENSE). This repository imports the application source, native Swift helpers, and required icon assets from the following baseline, with modifications by **elyann368**:

- Upstream version: `0.4.0`
- Upstream commit: [`50a94466dadef2c483913bc2cd5c76999cda4dda`](https://github.com/WizenPainter/shuffle/tree/50a94466dadef2c483913bc2cd5c76999cda4dda)
- [Original README at that commit](https://github.com/WizenPainter/shuffle/blob/50a94466dadef2c483913bc2cd5c76999cda4dda/README.md)
- Original icon assets: `AppIcon.icns` and `icon_base.png`; the current app reuses these assets.

Shuffle Flow 基于 **Jaime Guzman** 创建的 [WizenPainter/shuffle](https://github.com/WizenPainter/shuffle)，遵循 [MIT 许可证](LICENSE)。本仓库以以上 `0.4.0` 提交为基线，导入应用源码、Swift 原生辅助工具及构建所需的图标资源，由 **elyann368** 维护本地修改。当前应用图标沿用上游资源。原作者的版权声明与 MIT 许可全文均保留。

This is an independently maintained derivative project. The Shuffle Flow name and modifications do not imply endorsement by the upstream author. The main changes are described in [English](README.md) and [简体中文](README.zh-CN.md).

这是独立维护的衍生项目；Shuffle Flow 的名称和修改不代表上游作者背书。主要修改见 [English](README.md) 和 [简体中文](README.zh-CN.md)。

## Dependencies / 依赖

The app uses [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) for its Rust/Metal interface, alongside the crates declared in [Cargo.toml](Cargo.toml). Resolved versions are recorded in [Cargo.lock](Cargo.lock). Dependencies retain their own licenses; the project's MIT license does not replace them.

应用使用 [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) 构建 Rust/Metal 界面，其他依赖见 [Cargo.toml](Cargo.toml)，锁定版本见 [Cargo.lock](Cargo.lock)。各依赖遵循各自的许可证。
