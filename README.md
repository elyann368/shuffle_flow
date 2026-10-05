# Shuffle Flow

**English** | [简体中文](README.zh-CN.md)

A macOS file manager with a Chinese/English interface, improved Chinese filename input, and familiar Finder shortcuts. Built with Rust and GPUI/Metal.

Based on **[Shuffle by WizenPainter / Jaime Guzman](https://github.com/WizenPainter/shuffle)**, version `0.4.0`, at commit [`50a9446`](https://github.com/WizenPainter/shuffle/tree/50a94466dadef2c483913bc2cd5c76999cda4dda). Shuffle Flow is maintained by **elyann368** as an independent derivative. See [attribution](ATTRIBUTION.md) and the preserved [MIT license](LICENSE).

## What changed

| Area | Implemented changes |
| --- | --- |
| Interface | Simplified Chinese by default; live Chinese/English switching with a saved preference. Translates menus, settings, common actions, file types, and tag colors. Some secondary UI strings still need review. |
| Chinese names | Single-file/folder inline rename uses native macOS text input with composition and UTF-16 range handling for Chinese and emoji. New items enter this rename field. |
| Copy and move | Finder-style shortcuts and system file clipboard integration. ⌘X/⌘V cuts and moves local selections; Copy/paste supports multiple items and folders; moves do not overwrite existing destinations and report conflicts. |
| Archive extraction | ZIP and TAR variants extract into a separate sibling folder. Destination names avoid collisions; errors are shown and successful extraction can be undone. Available from the context menu, archive double-click, and a shortcut. |
| Tags menu | Root and submenu are measured separately. Opening a submenu keeps the root stationary; the submenu opens left when the right edge has insufficient space and avoids the bottom edge independently. This fixes the layout cycle that caused hover flicker. |
| Quick Look | Uses the native macOS panel. A single selection browses the displayed directory order; multiple selections browse only the chosen items. Up/Down navigate, Left/Right use native panel navigation, Space/Esc close, and browser focus follows. |
| Performance | Lightweight mode is on by default: no home-directory fuzzy index at launch. Bounded shared preview caches and limited background generation reduce accumulated memory during browsing. |
| Optional servers | SSH/SFTP is off by default, with a settings toggle. Disabling it blocks new connections and launch reconnection while preserving saved servers and credentials. |
| Paths and terminal | ⌘⇧T opens Terminal in the browsing directory; ⌘⇧D copies full paths; ⌘⇧F copies enclosing folder paths. Supports multiple selections, deduplicated parents, current-directory fallback, and configurable bindings. |
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

## Performance and connection settings

Open **Settings → General → Performance** (⌘,):

- **Lightweight mode:** on by default, intended for 8 GB Macs. Global name/content search uses Spotlight without a resident home-directory index. Direct path browsing and current-folder filtering remain available. Results depend on Spotlight coverage; excluded or unindexed folders may not appear.
- **Indexed search:** turn lightweight mode off to build a fuzzy index only when the command palette first opens. Estimated retained index data is limited to about 48 MiB, falling back to Spotlight when exceeded. It is released after the palette stays closed for two minutes. This is an estimate, not a whole-process hard limit.
- **Preview cache limit:** **16 MiB** by default; choose **8/16/32/64 MiB**. Thumbnails and PDF pages share this budget and a 64-result limit. Lowering it immediately evicts old images and their GPU cache entries. At most two previews are generated concurrently; PDF rasters have a maximum 800-pixel long edge.

The preview budget covers retained image data. Temporary decoding memory, GPUI/Metal, and native macOS Quick Look/services are outside this limit. Start with the defaults on an 8 GB Mac.

**Settings → General → Connections → Enable SSH / SFTP** is off by default. Enable it for server browsing and the existing per-server reconnect options. Disabling it blocks new connections and next-launch reconnection; close existing remote tabs after transfers finish. Saved servers, authentication preferences, and Keychain credentials are retained. System mounts such as SMB remain available. Upstream SSH already runs subprocesses on demand, so disabling it has little effect on baseline memory when no server is connected.

See [PERFORMANCE.md](PERFORMANCE.md) for measurement conditions and results.

## Keyboard shortcuts

| Action | Shortcut |
| --- | --- |
| Copy selected files | ⌘C |
| Cut selected files/folders | ⌘X |
| Paste into current folder (move after Cut) | ⌘V |
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
| Open Terminal in current folder | ⌘⇧T |
| Copy full path (Dir) | ⌘⇧D |
| Copy enclosing folder path (Folder) | ⌘⇧F |
| Open/close Quick Look | Space (Esc also closes) |
| Navigate Quick Look items | ↑/↓/←/→ |
| Settings | ⌘, |

Bindings can be changed in **Settings → Keybinds**. Existing explicitly cleared bindings remain cleared; use **Reset Keybinds to Default** if needed. While editing a name, ⌘C/⌘V operate on text. With a composing input method, confirm the candidate before confirming the rename.

D keeps the final item name; F removes it. For `/data/project/report.pdf`, D copies the full path and F copies `/data/project`. For a selected folder `/data/project`, D copies that folder and F copies `/data`. Multiple selections are sorted by path, one per line; F deduplicates parent directories. Without a selection, both copy the browsing directory. T uses the active pane's browsing directory, or the active column in Column view, regardless of selected folders. This version opens the system Terminal for local directories only; remote tabs can copy remote paths.

⌘X marks local files/folders for a move; ⌘V completes it in the browsing directory, including the active column. Sources remain intact until pasted. Multi-selection, progress, cancellation, and ⌘Z undo use the existing transfer engine. Existing destination names are skipped without overwriting; failed/skipped items remain pending for another destination. Successful items are removed from the cut batch, and repeated pastes are blocked while it is moving. Copying again (including in another app) cancels the old cut intent. Cut intent lasts only within the running Shuffle Flow process: pasting in Finder or restarting Shuffle Flow uses normal copy semantics. Remote cutting is not supported.

Saved custom bindings and explicit clears are preserved; new defaults do not take over saved bindings. If an older configuration has Copy Path unbound, assign ⌘⇧D individually. Resetting defaults also resets other custom bindings. Reassigning an occupied combination in Settings clears the previous action within the same input context.

## Core source layout

| File | Role |
| --- | --- |
| `src/main.rs` | File browser, file operations, shortcuts, native menus, and settings inherited from upstream and modified here. |
| `src/file_clipboard.rs` | Cut batch revisions, duplicate-paste protection, and partial-success retention. |
| `src/path_actions.rs` | Full/parent path handling, active-column directory selection, and literal terminal arguments. |
| `src/i18n.rs` | UI translation catalog and persisted language selection. |
| `src/ime.rs` | Inline rename's native text-input handler and Unicode/composition range conversion. |
| `src/quicklook.rs` | Native Quick Look controller, navigation, and browser focus synchronization. |
| `src/memory.rs` | Bounded LRU caches, Top-K search ranking, and regression tests. |
| `src/menu_layout.rs` | Independent root/submenu placement and regression tests. |
| `cloudctl.swift`, `removebg.swift` | Upstream native helpers for iCloud operations and image background removal. |
| `Cargo.toml`, `Cargo.lock` | Build options and locked Rust dependencies. |
| `make_app.sh`, `AppIcon.icns`, `icon_base.png` | App packaging and required upstream icon assets. |

## Verification

Before this sync, both the local-build and upstream-compatible configurations passed **17 tests** covering composition/range handling, Unicode Finder file URLs, Chinese filenames, copy/move conflicts, archive extraction, shortcuts, and menu placement. The release build and bundle signature checks passed.

The current version passes all **30 tests** with `cargo test --locked` and the default Shuffle Flow features. New coverage includes preview navigation boundaries, cache retention/eviction and immediate budget reduction, ranking equivalence, legacy preference migration, and saved performance/SSH settings. Cut tests cover clipboard replacement, duplicate-paste protection, partial failures, Unicode folders, preserved conflicts, cancellation, and move reversal. Path coverage includes Chinese/emoji names, deduplicated parents, root/empty-selection behavior, active-column paths, literal terminal arguments, and binding migration/conflicts.

Actual-window checks confirmed the Shuffle Flow menu name, left-opening/bottom-clamped tag submenu without moving the root, ⇧⌘N folder creation, and saving a pasted Chinese/emoji folder name. Native Quick Look was also checked across three Chinese-named text files: Down/Right/Up navigation, updated content, and Space to close. Column-view navigation was checked inside a child directory too. Native panel presentation is deferred outside GPUI updates to avoid reentrant App borrows while preview generators load. The performance/SSH controls were checked in the actual settings window, including persistence after restart. Pinyin candidate-window interaction has not yet been manually verified.

## Limitations and next improvements

- **Input methods:** native composition currently covers single-item inline rename. Search, batch rename, path editing, and other text fields still use upstream input handling; extend the handler and test them with Chinese input methods.
- **Translations:** review remaining secondary labels and messages; consolidate text resources to make bilingual maintenance easier.
- **Archives:** supported local formats are ZIP, TAR, TAR.GZ/TGZ, TAR.BZ2/TBZ, and TAR.XZ/TXZ. RAR, 7z, and encrypted-archive workflows need separate implementation.
- **Platform coverage:** test on Intel Macs and multiple macOS versions, including Finder clipboard interoperability and input-method candidate positioning.
- **Distribution:** add dedicated build CI and release signing/notarization for Shuffle Flow. The upstream release workflow and its signing credentials were not imported. Updates currently require rebuilding this repository.

These items are plans, not completed features. Bug reports are welcome through [this repository's Issues](https://github.com/elyann368/shuffle_flow/issues), with the macOS version and reproduction steps.

## License

[MIT](LICENSE). Original work © 2026 **Jaime Guzman**; Shuffle Flow modifications © 2026 **elyann368**. See [ATTRIBUTION.md](ATTRIBUTION.md) for the upstream baseline and asset/dependency credits.
