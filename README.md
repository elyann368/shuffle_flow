# Shuffle Flow

**English** | [简体中文](README.zh-CN.md)

A macOS file manager with a Chinese/English interface, improved Chinese filename input, and familiar Finder shortcuts. Built with Rust and GPUI/Metal.

Based on **[Shuffle by WizenPainter / Jaime Guzman](https://github.com/WizenPainter/shuffle)**, version `0.4.0`, at commit [`50a9446`](https://github.com/WizenPainter/shuffle/tree/50a94466dadef2c483913bc2cd5c76999cda4dda). Shuffle Flow is maintained by **elyann368** as an independent derivative. See [attribution](ATTRIBUTION.md) and the preserved [MIT license](LICENSE).

## What changed

| Area | Implemented changes |
| --- | --- |
| Interface | Simplified Chinese by default; live Chinese/English switching with a saved preference. Translates menus, settings, common actions, file types, and tag colors. Some secondary UI strings still need review. |
| Chinese names | Single-file/folder inline rename uses native macOS text input with composition and UTF-16 range handling for Chinese and emoji. New items enter this rename field. |
| Copy and move | Finder-style shortcuts and system file clipboard integration. Copy/paste supports multiple items and folders; moves do not overwrite existing destinations and report conflicts. |
| Archive extraction | ZIP and TAR variants extract into a separate sibling folder. Destination names avoid collisions; errors are shown and successful extraction can be undone. Available from the context menu, archive double-click, and a shortcut. |
| Tags menu | Root and submenu are measured separately. Opening a submenu keeps the root stationary; the submenu opens left when the right edge has insufficient space and avoids the bottom edge independently. This fixes the layout cycle that caused hover flicker. |
| Branding and updates | The packaged app and native app menu use **Shuffle Flow**. Local builds have separate preferences and disable the upstream binary updater to retain these changes. |

Inherited from Shuffle: tabs, split panes, a command palette, previews, themes, and cloud/server browsing. These are upstream features; this change set does not reimplement or fully revalidate all of them.

## Build and run

Validated on **Apple Silicon macOS**. Install Rust and Xcode Command Line Tools first. Intel, Linux, and Windows builds have not been validated for this fork.

```sh
git clone https://github.com/elyann368/shuffle_flow.git
cd shuffle_flow
./make_app.sh
open "Shuffle Flow.app"
```

The script builds a release executable, packages the icon, compiles the optional native Swift helpers when available, and applies an ad-hoc signature. It does not perform Apple notarization. Generated app bundles, archives, build caches, and local settings are excluded from Git.

The default Cargo features are `runtime-shaders` and `local-build`: Metal shaders compile at runtime, so a standalone Metal compiler is not required; the app keeps the Shuffle Flow name and does not install upstream updates.

```sh
cargo run --locked
cargo test --locked
cargo build --locked --release
# Faster development app bundle:
SHUFFLE_PROFILE=debug ./make_app.sh
```

Settings: **Settings → General → Explorer → Language**. Application identifier: `com.shuffle.local.zh`; preferences: `~/Library/Application Support/Shuffle-zh/`. These identifiers are retained for continuity with the earlier local build. To use an isolated test configuration:

```sh
SHUFFLE_CONFIG_DIR=/tmp/shuffle-flow-config cargo run --locked
```

## Keyboard shortcuts

| Action | Shortcut |
| --- | --- |
| Copy selected files | ⌘C |
| Paste into current folder | ⌘V |
| Move copied files here | ⌥⌘V |
| New folder | ⇧⌘N |
| Rename selection | Return |
| Open selection / extract selected archive | ⌘↓ |
| Parent folder | ⌘↑ |
| Duplicate | ⌘D |
| Move to Trash | ⌘⌫ |
| Extract selected archive | ⌥⌘E (added by this fork) |
| Undo file operation | ⌘Z |
| Get Info | ⌘I |
| Quick Look | Space |
| Settings | ⌘, |

Bindings can be changed in **Settings → Keybinds**. Existing explicitly cleared bindings remain cleared; use **Reset Keybinds to Default** if needed. While editing a name, ⌘C/⌘V operate on text. With a composing input method, confirm the candidate before confirming the rename.

## Core source layout

| File | Role |
| --- | --- |
| `src/main.rs` | File browser, file operations, shortcuts, native menus, and settings inherited from upstream and modified here. |
| `src/i18n.rs` | UI translation catalog and persisted language selection. |
| `src/ime.rs` | Inline rename's native text-input handler and Unicode/composition range conversion. |
| `src/menu_layout.rs` | Independent root/submenu placement and regression tests. |
| `cloudctl.swift`, `removebg.swift` | Upstream native helpers for iCloud operations and image background removal. |
| `Cargo.toml`, `Cargo.lock` | Build options and locked Rust dependencies. |
| `make_app.sh`, `AppIcon.icns`, `icon_base.png` | App packaging and required upstream icon assets. |

## Verification

Before this sync, both the local-build and upstream-compatible configurations passed **17 tests** covering composition/range handling, Unicode Finder file URLs, Chinese filenames, copy/move conflicts, archive extraction, shortcuts, and menu placement. The release build and bundle signature checks passed.

After importing the core source, `cargo test --locked` in this repository also passed all **17 tests** with the default Shuffle Flow features.

Actual-window checks confirmed the Shuffle Flow menu name, left-opening/bottom-clamped tag submenu without moving the root, ⇧⌘N folder creation, and saving a pasted Chinese/emoji folder name. Pinyin candidate-window interaction has not yet been manually verified.

## Limitations and next improvements

- **Input methods:** native composition currently covers single-item inline rename. Search, batch rename, path editing, and other text fields still use upstream input handling; extend the handler and test them with Chinese input methods.
- **Translations:** review remaining secondary labels and messages; consolidate text resources to make bilingual maintenance easier.
- **Archives:** supported local formats are ZIP, TAR, TAR.GZ/TGZ, TAR.BZ2/TBZ, and TAR.XZ/TXZ. RAR, 7z, and encrypted-archive workflows need separate implementation.
- **Platform coverage:** test on Intel Macs and multiple macOS versions, including Finder clipboard interoperability and input-method candidate positioning.
- **Distribution:** add dedicated build CI and release signing/notarization for Shuffle Flow. The upstream release workflow and its signing credentials were not imported. Updates currently require rebuilding this repository.

These items are plans, not completed features. Bug reports are welcome through [this repository's Issues](https://github.com/elyann368/shuffle_flow/issues), with the macOS version and reproduction steps.

## License

[MIT](LICENSE). Original work © 2026 **Jaime Guzman**; Shuffle Flow modifications © 2026 **elyann368**. See [ATTRIBUTION.md](ATTRIBUTION.md) for the upstream baseline and asset/dependency credits.
