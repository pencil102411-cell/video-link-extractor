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
- Rust 单元测试已覆盖 URL 边界、质量档位白名单和进度解析；因缺少 Windows C++ 链接器，测试和 Tauri 打包尚未能实际链接执行。
