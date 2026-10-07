# 开发记录

## 2026-10-05

- 完成 GitHub 同类项目调研，确认采用 yt-dlp + FFmpeg，不重复实现站点解析。
- 按用户确认的方案创建 Windows x64 Tauri 2 + Vue 3 项目。
- 已实现方向：单条 URL、固定质量档位、本地输出、依赖自检、解析元数据、下载进度、取消和重试。
- 当前环境已安装 Node.js/npm 和 Rust 1.99；本机仍缺 Visual Studio C++ Build Tools 的 `link.exe`，因此最终 Windows 打包暂未完成。
- `tools/` 已放入 yt-dlp 2026.08.19、FFmpeg/FFprobe Windows x64 构建和 Deno 2.9.7；二进制被 `.gitignore` 排除，更新时按 `tools/README.md` 替换。

## 2026-10-06

- 根据代码审查补上了必需运行时门禁、单任务并发边界、Windows 进程树取消、完成文件白名单和启动阶段事件监听顺序。
- URL 校验增加 URL 解析、IPv4-mapped IPv6 和 DNS 解析结果检查；Tauri WebView 增加最小 CSP，远程封面仅允许 HTTPS 图片。
- 下载进度增加 FFmpeg 后处理阶段状态，界面会显示“正在合并视频”。
- Rust 单元测试已覆盖 URL 边界、质量档位白名单和进度解析；Visual Studio Build Tools 已补齐，`cargo test` 通过。
- 修复打包后运行时资源路径：Tauri 资源显式映射到安装目录 `tools/`，并兼容旧版 `_up_/tools`。
- 继续兼容 Windows 安装版的 `resource_dir/resources` 布局，运行时会回退查找 exe 同级和安装目录下的 `tools/`。
- 修复运行时误判：FFmpeg/FFprobe 对 `--version` 返回非零，现会回退尝试 `-version`。
- 修复 Windows 下载目录参数：yt-dlp 的 `--paths` 使用 `home:<目录>` 语法，避免把 `home=...` 当成目录名。
- 使用用户提供的 `https://xhslink.cn/o/2KqnhGIdSrI` 完成真实验收：短链解析、重定向地址提取、MP4 下载和 ffprobe 校验均成功；输出为 H.264/AAC、1280×720、约 255 秒、28.8 MB。
- 修复小红书短链跳转登录/404 包装页：从 yt-dlp 错误中的 `redirectPath` 提取同站公开内容地址并重试；需要登录或链接失效时显示明确提示，下载使用解析后的地址。
- 支持直接粘贴带说明文字的分享内容：后端提取首个 `http(s)` 链接后再校验和解析，前端输入框改为文本模式并在输入变化时清除旧解析结果。
- `npm run check`、Rust 12 项单元测试和 `npm run tauri:build` 均通过。
- 最新 Windows x64 安装包：`src-tauri/target/release/bundle/msi/video-link-extractor_0.1.1_x64_en-US.msi`、`src-tauri/target/release/bundle/nsis/video-link-extractor_0.1.1_x64-setup.exe`。
