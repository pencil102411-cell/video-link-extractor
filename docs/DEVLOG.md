# 开发记录

## 2026-10-05

- 完成 GitHub 同类项目调研，确认采用 yt-dlp + FFmpeg，不重复实现站点解析。
- 按用户确认的方案创建 Windows x64 Tauri 2 + Vue 3 项目。
- 已实现方向：单条 URL、固定质量档位、本地输出、依赖自检、解析元数据、下载进度、取消和重试。
- 当前环境已安装 Node.js/npm 和 Rust 1.99；正在安装 Visual Studio C++ Build Tools，以完成最终 Windows 打包。
- `tools/` 已放入 yt-dlp 2026.08.19 与 FFmpeg/FFprobe Windows x64 构建；二进制被 `.gitignore` 排除，更新时按 `tools/README.md` 替换。
