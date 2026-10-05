# 本地视频链接提取工具：开源项目调研与落地方案

调研日期：2026-10-05

## 1. 需求边界

目标是一个只给本人使用的本地工具：粘贴视频页面链接，提取并保存为本地视频文件。工具本身不对公网提供服务，不做多用户、账号体系、远程 API 或 SaaS 部署。

“本地使用”不等于“下载过程不联网”：界面、队列、历史、临时文件和最终视频留在本机；解析页面、访问媒体流、使用登录 Cookie，以及更新提取器时仍需要访问目标站点或更新源。

## 2. 结论先行

不要重新实现站点解析、格式识别、音视频合并。成熟项目都把 `yt-dlp` 作为核心提取/下载引擎，把 `FFmpeg` 作为分离音视频合并和后处理工具。

建议开发一个“薄本地桌面壳”，首版只覆盖：单条 URL、解析信息、选择视频质量、下载队列、进度、取消/重试、输出目录和错误日志。底层直接复用 `yt-dlp`，随应用携带 `ffmpeg/ffprobe`；不复制 MeTube 的订阅体系，也不复制 Cobalt 的代理 API。

默认实现建议：

- **界面形态**：桌面应用，无 HTTP 监听；默认先做 Windows x64。
- **桌面技术**：Tauri 2 + Vue 3 + TypeScript；Rust 主进程负责任务、文件和子进程，前端只通过受限 IPC 调用。
- **下载引擎**：随包携带固定版本的 `yt-dlp.exe`，通过参数数组启动，不拼接 shell 命令；后续如需要跨平台，再加入 macOS/Linux 二进制。
- **媒体处理**：随包携带 `ffmpeg.exe` 和 `ffprobe.exe`；优先无损合并，无法合并为 MP4 时明确提示或回退 MKV。
- **状态存储**：首版用本地 JSON/SQLite 二选一；单条下载先用 JSON 即可，只有要搜索历史和批量任务时再上 SQLite。

如果只追求最快可用，也可以不开发新工具，直接试用 [Stacher](https://github.com/Lacnard/stacher)（Windows 本地 Electron）或 [Open Video Downloader](https://github.com/jely2002/youtube-dl-gui)（跨平台 Tauri）。自研的价值应放在更少的选项、更贴合你的工作流和后续 AI 视频素材整理，而不是重做下载器。

## 3. 开源项目对比

| 项目 | 功能与架构 | 适合之处 | 主要不足/风险 |
|---|---|---|---|
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | Python 提取器/插件体系；CLI 与 Python API；支持数千站点；提供 Windows/macOS/Linux 独立可执行文件。官方 README 明确 `ffmpeg/ffprobe` 用于音视频合并，完整 YouTube 支持还建议 `yt-dlp-ejs` 与 JS runtime。 | 覆盖面、格式选择、站点适配和更新能力最强，应该作为底层引擎。 | 自己没有 UI；站点变化、地区限制、登录和反爬无法由上层完全消除；依赖 `ffmpeg` 和部分站点所需的 JS runtime 增加打包工作。稳定版可能落后于站点变化，需保留手动更新通道。 |
| [Open Video Downloader](https://github.com/jely2002/youtube-dl-gui) | Vue 3 + TypeScript 前端，Rust + Tauri 2 后端；调用 yt-dlp；支持视频/音频、字幕、元数据、播放列表、格式和输出模板、Cookie、队列与通知。 | 与目标形态最接近，跨平台安装包和 IPC 边界值得借鉴。 | AGPL-3.0；功能已经偏通用下载器，二次改造成本高；仓库依赖 Sentry，若用于私有工具需先审查并关闭遥测；自动更新会产生额外外联。 |
| [Stacher](https://github.com/Lacnard/stacher) | Electron + React + TypeScript；主进程管理队列、下载器、凭据和 SQLite 历史，renderer 通过 preload/contextBridge 调用；把 yt-dlp/ffmpeg 放入 resources。 | 最贴近“本机粘贴 URL → 下载”，无 HTTP 端口；队列、重试、历史、Cookie 和打开文件夹的产品细节完整。 | 当前面向 Windows 10/11 x64；Electron 包体较大；暂停实际是取消后续传，不是真正暂停；项目仍较新，成熟度要单独验证。 |
| [MeTube](https://github.com/alexta69/metube) | Python 3.13+、aiohttp、Socket.IO、yt-dlp 后端；Angular 22 前端；JSON 状态存储；官方 Docker 多架构镜像；有队列、订阅、频道/播放列表、格式、目录和 yt-dlp 预设。 | 队列、实时进度、路径清洗、状态持久化和安全边界经验丰富。 | Web 服务对单机需求偏重；默认监听 `0.0.0.0:8081`，本机使用必须改为 `127.0.0.1`；AGPL-3.0；任意 yt-dlp 参数覆盖可能变成命令执行入口。 |
| [yt-dlp-web](https://github.com/sjtrny/yt-dlp-web) | 简单 Web UI + Docker；支持视频/直播/播放列表、并发上限、状态历史和任务；默认可限制到 loopback。 | 适合作为极简队列、断点和重启行为的参考。 | 服务端模型仍比桌面单机复杂；重启后未完成任务会变为 interrupted，不自动恢复；功能和工程生态不如核心项目成熟。 |
| [Media Downloader](https://github.com/mhogomchungu/media-downloader) | Qt/C++ GUI，默认 yt-dlp，也可扩展 gallery-dl、aria2 等；支持批量、并发、播放列表、订阅和多语言；GPL-2.0。 | 证明“GUI 只是多个 CLI 工具的编排层”这条路线可行，Windows 有便携包。 | 范围过大，支持很多不需要的下载器；默认会联网拉取 yt-dlp/扩展；GPL-2.0；对单链接工作流显得臃肿。 |
| [Tartube](https://github.com/axcore/tartube) | Python 3 + GTK 3 桌面 GUI；频道/播放列表数据库、直播提醒、归档、目录组织、带宽/并发控制；LGPL-2.1。 | 数据库、归档、长期素材管理经验成熟。 | GTK/Python 依赖和打包复杂；功能远超单条 URL 提取，不适合拿来做首版底座。 |
| [cobalt](https://github.com/imputnet/cobalt) | Node/Express API + Web 前端的代理式下载服务；AGPL-3.0；面向公开内容，不缓存内容；支持站点和输出能力按站点变化。 | 粘贴链接的简洁 UX、公开部署的限流和安全思路值得参考。 | 代理/API/容器化架构与本机单用户不匹配；需要维护 Node、API、前端和服务安全；不应作为本地底座。 |

## 4. 从已有项目提炼出的工程经验

### 已验证的事实

1. `yt-dlp` 支持数千站点，并同时提供 CLI、Python API 和独立二进制；官方文档建议通过 API 的 `YoutubeDL`、`progress_hooks` 等接口集成，而不是依赖普通 stdout 文本。
2. 高质量视频常常是分离的视频流和音频流，`ffmpeg/ffprobe` 是合并与后处理的必要依赖；完整 YouTube 支持还涉及 `yt-dlp-ejs` 与 Deno/Node/Bun/QuickJS 等 JS runtime。
3. 站点支持不是永久承诺。官方 supported sites 文档明确指出，网站变化会造成提取器失效，最终仍要以实际测试为准。
4. Web UI 默认监听所有网卡是实际暴露风险：MeTube 和部分 yt-dlp Web UI 都需要显式改成 `127.0.0.1` 才符合本机私有边界。
5. Cookie、URL、视频标题、播放列表名称和站点返回的元数据都是不可信输入；文件名和输出路径必须做清洗和目录越界检查。
6. yt-dlp 源码/PyPI 代码使用 Unlicense，但部分预构建二进制包含其他许可证组件；FFmpeg 的默认部分是 LGPL，启用 GPL 部分后许可证会变化。分发时需要保留许可证与第三方声明。

### 工程取舍

- **用 CLI 还是 Python API**：首版推荐随包携带官方 `yt-dlp.exe`，由 Rust 主进程启动并用结构化 JSON/进度模板接收状态；这样不需要再打包 Python。若后续需要更细的进度回调或复杂后处理，再切到 Python API。
- **用桌面还是本地 Web**：桌面无监听端口，天然符合“不对外使用”；本地 Web 只有在需要浏览器访问、手机访问或后续远程控制时才值得引入。
- **要不要数据库**：单条下载和最近记录用 JSON 足够；如果一开始就上 SQLite、Socket.IO、订阅和迁移，会把下载器问题变成媒体管理系统问题。
- **是否开放任意 yt-dlp 参数**：首版不开放。应把少量安全选项映射为固定 profile，避免 shell 注入、路径穿越和难以复现的失败。

## 5. 建议的产品范围

### MVP（第一版）

- 粘贴一条视频页面 URL。
- 点击“解析”，显示标题、封面、时长、可用最高分辨率和文件大小（能取得时显示）。
- 选择三个固定档位：`最佳可用`、`1080p 优先`、`720p 优先`；默认优先 MP4/H.264/AAC，保留画质，不主动转码。
- 点击“下载”，保存到用户选择的目录；默认目录为应用旁的 `downloads/`。
- 显示进度、速度、ETA、阶段（解析/下载/合并/完成）。
- 支持取消、失败重试、已有 `.part` 文件续传。
- 完成后提供“打开文件”和“打开所在文件夹”。
- 记录 URL、标题、时间、状态、最终路径和错误摘要；不记录账号密码。

### 后置功能

播放列表/频道订阅、批量导入、字幕、缩略图、SponsorBlock、浏览器 Cookie 管理、代理、任务计划、片段裁剪、转码和素材库整理都放到第二阶段，必须以真实使用频率决定是否加入。

## 6. 推荐的本地架构

```text
Vue 3 UI
   │ 受限 Tauri IPC
Rust 主进程 / Task Manager
   ├─ URL 校验与固定 profile 映射
   ├─ yt-dlp.exe 子进程
   ├─ ffmpeg.exe / ffprobe.exe
   ├─ 进度、取消、重试、断点状态
   └─ 本地 JSON/SQLite + 日志
本机 downloads/ 输出目录
```

关键约束：

- 只接受 `http`/`https` URL；不接受 `file://`、本地网段或任意内部地址。
- 所有子进程用参数数组启动，不经过 shell；固定 `--no-playlist`，首版只下载单条视频。
- 输出模板使用清洗后的标题并追加视频 ID，避免重名和路径穿越。
- 只把必要字段发送给前端；Cookie 文件不进入日志，不上传，不放进 renderer。
- 默认不开自动更新和遥测；提供“检查 yt-dlp 版本”按钮，由用户手动触发。
- 默认并发 1，后续根据实际网络和账号风控再开放 2。
- 打包时锁定 yt-dlp、FFmpeg 和 JS runtime 版本，附带 LICENSE/THIRD_PARTY_NOTICES。

## 7. 分阶段开发

1. **验证阶段**：先用一两个公开、用户有权下载的链接验证 yt-dlp + FFmpeg 能否取得预期 MP4；同时确认目标平台是否需要登录 Cookie。
2. **MVP 阶段**：完成单 URL、解析、下载、进度、取消、重试、输出目录和错误日志；只打 Windows x64 包。
3. **稳定性阶段**：补齐断点、异常退出恢复、重复文件策略、依赖自检、版本锁定和安装包签名/校验。
4. **扩展阶段**：依据真实工作流再决定是否加入批量、字幕、Cookie、代理和素材整理。

## 8. 需要确认的决策

我建议按以下默认值进入开发：**Windows x64、Tauri 2 + Vue 3、单条视频、桌面应用、`yt-dlp.exe + ffmpeg.exe` 随包、无 HTTP 服务（若未来增加 Web 则只绑定 `127.0.0.1`）、首版不做登录和批量**。确认后再开始搭建项目。


## 9. 一手来源索引

- yt-dlp：[README.md](https://github.com/yt-dlp/yt-dlp/blob/master/README.md)、[supported sites](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md)、[pyproject.toml](https://github.com/yt-dlp/yt-dlp/blob/master/pyproject.toml)。重点核对独立二进制、更新通道、FFmpeg/JS runtime 依赖、格式选择、`--no-playlist`、嵌入 API 和许可证。
- Open Video Downloader：[README.md](https://github.com/jely2002/youtube-dl-gui/blob/main/README.md)、[package.json](https://github.com/jely2002/youtube-dl-gui/blob/main/package.json)、[src-tauri/Cargo.toml](https://github.com/jely2002/youtube-dl-gui/blob/main/src-tauri/Cargo.toml)。
- Stacher：[README.md](https://github.com/Lacnard/stacher/blob/master/README.md)、[package.json](https://github.com/Lacnard/stacher/blob/master/package.json)。
- MeTube：[README.md](https://github.com/alexta69/metube/blob/master/README.md)、[AGENTS.md](https://github.com/alexta69/metube/blob/master/AGENTS.md)、[app/ytdl.py](https://github.com/alexta69/metube/blob/master/app/ytdl.py)。
- yt-dlp-web：[README.md](https://github.com/sjtrny/yt-dlp-web/blob/main/README.md)。
- Media Downloader：[README.md](https://github.com/mhogomchungu/media-downloader/blob/master/README.md)。
- Tartube：[README.rst](https://github.com/axcore/tartube/blob/master/README.rst)。
- Cobalt：[README.md](https://github.com/imputnet/cobalt/blob/main/README.md)、[api/README.md](https://github.com/imputnet/cobalt/blob/main/api/README.md)、[api/package.json](https://github.com/imputnet/cobalt/blob/main/api/package.json)。
- FFmpeg：[LICENSE.md](https://github.com/FFmpeg/FFmpeg/blob/master/LICENSE.md) 和 [官方法律说明](https://ffmpeg.org/legal.html)。
