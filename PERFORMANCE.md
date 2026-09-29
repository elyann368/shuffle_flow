# Performance notes / 性能说明

## Changes / 改动

The main baseline-memory change is removing the eager home-directory index. Lightweight mode (default) uses Spotlight for global search. Indexed mode is opt-in, builds lazily, estimates retained data against a 48 MiB budget, falls back when exceeded, and releases the index after two idle minutes with the palette closed. Search ranking retains only the best candidates instead of cloning/sorting the entire index.

基础占用的主要改动是取消启动时建立个人目录索引。默认轻量模式使用 Spotlight 全局搜索；关闭轻量模式后才按需建立模糊索引，以约 48 MiB 预算限制保留数据，超出后回退；命令面板关闭闲置两分钟后释放索引。搜索只保留排名靠前的候选，避免复制或排序整份索引。

Thumbnail and PDF caches share a configurable 8/16/32/64 MiB retained-image budget (default 16), with a 64-result cap and explicit GPU-cache eviction. Failed previews count toward the entry cap. Generation is deduplicated and limited to two concurrent tasks; PDF pages are mapped where possible and rasterized to at most 800 pixels per edge. File-info and PDF-count caches are capped at 512 and 256 entries.

缩略图和 PDF 页面共享可调的 8／16／32／64 MiB 图像预算（默认 16），最多缓存 64 个结果，淘汰时释放相应 GPU 缓存。失败结果也占条目名额。预览任务去重且最多同时执行两个，PDF 尽可能映射读取，栅格每边不超过 800 像素。文件信息和 PDF 页数缓存分别最多保留 512、256 条。

SSH/SFTP is disabled by default and does not reconnect saved servers until enabled. It already uses on-demand system processes; the toggle does not remove compiled code or terminate ongoing transfers. Close existing remote tabs when finished.

SSH/SFTP 默认关闭，启用后才允许服务器连接和启动重连。它原本使用按需启动的系统进程；开关不会移除已编译的代码，也不会终止正在传输的任务。已有远程标签页请在操作完成后关闭。

## Local startup sample / 本机启动采样

Measured on 2026-09-29: Apple Silicon, 8 GB RAM, macOS 27.0 (26A428), release build. Separate fresh configurations opened the same directory containing three small Chinese-named text files, with inspector preview off and no SSH connections. The baseline was the previous Shuffle Flow release with eager indexing, not unmodified upstream Shuffle.

2026-09-29 在 Apple Silicon、8 GB 内存、macOS 27.0（26A428）机器上测量 release 构建。两个独立的新配置打开同一测试目录（3 个中文名称的小文本文件），检查器预览关闭，无 SSH 连接。基线为此前保留启动索引行为的 Shuffle Flow 构建，并非未经修改的上游版本。

| Metric / 指标 | Previous build / 修改前 | Optimized build / 优化后 |
| --- | ---: | ---: |
| Physical footprint (`vmmap -summary`) / 实际物理占用 | 196.7 MiB | 73.4 MiB |
| Peak observed footprint / 采样时已记录的峰值 | 225.1 MiB | 105.8 MiB |
| Resident set (`ps`, KiB) / 驻留集 | 98,032 | 90,928 |

An earlier optimized-build sample was 61.6 MiB (peak 95.0 MiB); the table records the later build with performance controls. Each value is a single startup/idle sample, taken roughly 10–16 seconds after launch, before opening Quick Look or the command palette. The samples are indicative, not a repeated benchmark or a guarantee. Resource caches, directory sizes, active tabs, and system memory pressure affect results. No Finder comparison was measured; these numbers do not establish lower memory than Finder.

优化后的较早构建另一次采样为 61.6 MiB（峰值 95.0 MiB）；表格记录加入性能设置后的构建。以上各为启动后约 10–16 秒的一次闲置采样，尚未打开快速查看或命令面板。它们只用于说明这一测试条件下的改善，并非重复基准测试或占用承诺。系统资源缓存、目录规模、标签页及内存压力都会影响结果。尚未测量 Finder 对照，因此不能据此认定比 Finder 更省内存。

To reproduce, build the previous and current revisions in release mode, run each with a distinct `SHUFFLE_CONFIG_DIR`, set both to the same test directory, and record `ps -p <pid> -o rss=,%cpu=,etime=` and `vmmap -summary <pid>` after the same idle interval. Compare physical footprint separately from RSS; they are different accounting metrics. Avoid publishing raw process maps, which may include personal paths.

复测时分别构建前后版本，使用独立的 `SHUFFLE_CONFIG_DIR`，打开同一目录并等待相同时间，再使用以上 `ps` 和 `vmmap` 命令记录。实际物理占用与 RSS 的统计口径不同，请分别比较；原始进程映射可能含个人路径，不应直接公开。

## Limits / 边界

Cache and index budgets govern retained application data, not total app memory. Temporary decode buffers, GPU/framework resources, currently displayed directory entries, native Quick Look and external services can add memory. Large-folder browsing, long-running sessions, cloud placeholders, and active remote transfers need further workload measurements. When lightweight mode is on, Spotlight exclusions and incomplete indexing can limit global-search results.

缓存和索引预算限制应用保留的数据，不是进程总内存上限。临时解码缓冲、GPU／框架资源、当前目录条目、原生快速查看及外部服务仍会占用内存。大型目录、长时间连续使用、云端占位文件和远程传输仍需进一步测量。轻量模式的全局搜索结果受到 Spotlight 排除范围及索引完整性的影响。
