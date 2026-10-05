# Runtime tools

将以下官方运行时放入本目录后，桌面应用即可执行下载：

- `yt-dlp.exe`：从 https://github.com/yt-dlp/yt-dlp/releases/latest 下载 Windows x64 独立版。
- `ffmpeg.exe` 与 `ffprobe.exe`：从 https://github.com/yt-dlp/FFmpeg-Builds/releases 获取 Windows 构建并放入本目录。
- `deno.exe`（可选）：部分 YouTube 场景需要，用于 yt-dlp-ejs 的 JavaScript runtime。

开发模式和发布包都只使用本目录（或打包后的应用资源目录）中的文件，不回退到系统 PATH，避免误执行同名外部程序。

请同时保留各运行时的许可证与第三方声明，发布时不要把这些工具的许可证和主应用许可证混为一谈。

当前开发目录使用的是 yt-dlp 官方 Windows 独立版，以及 `yt-dlp/FFmpeg-Builds` 的 GPL Windows x64 构建。若对外分发安装包，需要同时遵守 yt-dlp、FFmpeg 及其构建所带许可证；本工具本身不替代这些项目的许可证声明。

当前目录还包含 Deno 2.9.7 Windows x64，用于 yt-dlp-ejs 的 JavaScript runtime，提升 YouTube 等站点的解析成功率。
