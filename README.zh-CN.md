# Shuffle Flow

[English](README.md) | **简体中文**

面向 macOS 的文件管理器，提供中英文界面，改善中文文件名输入，并补齐熟悉的 Finder 快捷键。使用 Rust 与 GPUI/Metal 构建。

基于 **[WizenPainter / Jaime Guzman 的 Shuffle](https://github.com/WizenPainter/shuffle)**，以上游 `0.4.0`、提交 [`50a9446`](https://github.com/WizenPainter/shuffle/tree/50a94466dadef2c483913bc2cd5c76999cda4dda) 为基线，由 **elyann368** 独立维护。完整来源说明见 [ATTRIBUTION.md](ATTRIBUTION.md)，原作者版权声明与 [MIT 许可证](LICENSE) 均已保留。

## 修改与完善说明

| 范围 | 已完成的改动 |
| --- | --- |
| 中文界面 | 默认简体中文，可在设置中立即切换中文／English，并记住选择。翻译菜单、设置、常用操作、文件类型与标签颜色；部分次要界面文字仍需检查。 |
| 中文命名 | 文件／文件夹的单项重命名接入 macOS 原生输入法，处理组合文字及 UTF-16 范围，支持中文和 emoji。新建项目自动进入该重命名框。 |
| 复制与移动 | 使用 Finder 风格快捷键和系统文件剪贴板，支持多个项目与文件夹。移动不会覆盖已有目标，冲突会显示原因。 |
| 解压 | 支持本地 ZIP 与 TAR 系列，解压到压缩包旁的独立文件夹，自动避开重名。失败显示原因，成功后可撤销；支持右键菜单、双击压缩包及快捷键。 |
| 标签菜单闪烁 | 主菜单与子菜单分别测量和定位。展开子菜单时主菜单保持原位；右侧空间不足时向左展开，底部避让也独立完成，修复布局来回移动导致的悬停闪烁。 |
| 命名与更新 | 应用及原生菜单统一为 **Shuffle Flow**。本地构建使用独立配置，并关闭上游二进制自动更新，以保留本仓库的修改。 |

标签页、双栏、命令面板、预览、主题和云盘／服务器浏览等能力继承自 Shuffle。这些属于上游功能，本次修改没有重新实现或全面验证所有继承功能。

## 构建与运行

目前在 **Apple Silicon macOS** 上验证。需要先安装 Rust 与 Xcode Command Line Tools。本分支尚未验证 Intel、Linux 或 Windows 构建。

```sh
git clone https://github.com/elyann368/shuffle_flow.git
cd shuffle_flow
./make_app.sh
open "Shuffle Flow.app"
```

脚本构建 release 程序、打包图标，在可用时编译 Swift 原生辅助工具，并进行临时签名（ad-hoc），尚未进行 Apple 公证。生成的应用、压缩包、构建缓存和个人设置不提交到 Git。

Cargo 默认开启 `runtime-shaders` 与 `local-build`：Metal 着色器在运行时编译，无需独立 Metal 编译器；应用默认使用 Shuffle Flow 名称，并保留本地修改而不安装上游更新。

```sh
cargo run --locked
cargo test --locked
cargo build --locked --release
# 开发时生成 debug 应用包：
SHUFFLE_PROFILE=debug ./make_app.sh
```

语言设置位于 **设置 → 通用 → 文件浏览 → 语言**。应用标识为 `com.shuffle.local.zh`，配置目录为 `~/Library/Application Support/Shuffle-zh/`；沿用早期本地构建的标识，以保持设置连续性。测试时可使用独立配置：

```sh
SHUFFLE_CONFIG_DIR=/tmp/shuffle-flow-config cargo run --locked
```

## 常用快捷键

| 操作 | 按键 |
| --- | --- |
| 复制所选项目 | ⌘C |
| 粘贴到当前文件夹 | ⌘V |
| 将复制的项目移到当前文件夹 | ⌥⌘V |
| 新建文件夹 | ⇧⌘N |
| 重命名所选项目 | Return |
| 打开所选项目／解压所选压缩包 | ⌘↓ |
| 返回上级文件夹 | ⌘↑ |
| 制作副本 | ⌘D |
| 移到废纸篓 | ⌘⌫ |
| 解压所选压缩包 | ⌥⌘E（本分支新增） |
| 撤销文件操作 | ⌘Z |
| 显示简介 | ⌘I |
| 快速查看 | 空格 |
| 设置 | ⌘, |

可在 **设置 → 快捷键** 修改绑定。旧设置里主动清空过的绑定继续保留；需要时点击 **恢复默认快捷键**。编辑名称时，⌘C／⌘V 操作文字。使用组合输入法时，先确认候选，再确认重命名。

## 核心代码结构

| 文件 | 作用 |
| --- | --- |
| `src/main.rs` | 文件浏览、文件操作、快捷键、原生菜单与设置，包含上游实现及本次修改。 |
| `src/i18n.rs` | 界面翻译表与语言选择持久化。 |
| `src/ime.rs` | 单项重命名的原生文字输入、Unicode 范围转换及组合输入处理。 |
| `src/menu_layout.rs` | 主菜单与子菜单的独立定位及回归测试。 |
| `cloudctl.swift`、`removebg.swift` | 上游 iCloud 操作与图片背景移除的原生辅助工具。 |
| `Cargo.toml`、`Cargo.lock` | 构建选项与锁定的 Rust 依赖。 |
| `make_app.sh`、`AppIcon.icns`、`icon_base.png` | 应用打包及构建所需的上游图标资源。 |

## 验证记录

同步前，本地修改模式与兼容上游的构建模式各通过 **17 项测试**，覆盖组合输入／字符范围、Finder Unicode 文件 URL、中文文件名、复制／移动冲突、解压、快捷键及菜单定位。release 构建和应用签名检查通过。

核心源码导入本仓库后，使用默认 Shuffle Flow 功能运行 `cargo test --locked`，**17 项测试全部通过**。

实际窗口中已确认 Shuffle Flow 菜单名称、标签子菜单在右侧向左展开及在底部独立避让时主菜单保持原位、⇧⌘N 新建文件夹，以及粘贴中文／emoji 名称后保存。拼音输入法候选框交互尚未完成人工验证。

## 当前限制与后续完善

- **输入法：** 原生组合输入目前接入单项重命名。搜索、批量重命名、路径编辑等输入栏仍沿用上游实现，后续需扩展输入处理并逐项测试中文输入法。
- **翻译：** 检查遗漏的次要标签与提示，进一步整理文字资源，降低中英文维护成本。
- **压缩格式：** 已支持 ZIP、TAR、TAR.GZ／TGZ、TAR.BZ2／TBZ、TAR.XZ／TXZ；RAR、7z 与加密压缩包操作仍需单独实现。
- **平台验证：** 补充 Intel 和不同 macOS 版本的测试，尤其是 Finder 文件剪贴板互通与输入法候选框定位。
- **发布：** 建立 Shuffle Flow 专用构建 CI，完善正式签名与 Apple 公证。未导入上游发布工作流或签名凭据，目前更新需重新构建本仓库。

以上是后续计划，不是已完成的功能。欢迎在 [本仓库 Issues](https://github.com/elyann368/shuffle_flow/issues) 提交问题，并提供 macOS 版本与复现步骤。

## 许可证与引用

采用 [MIT 许可证](LICENSE)。原项目 © 2026 **Jaime Guzman**；Shuffle Flow 修改 © 2026 **elyann368**。上游基线、沿用的图标与依赖说明见 [ATTRIBUTION.md](ATTRIBUTION.md)。
