# Video Link Extractor

一个只在本机运行的桌面工具：粘贴视频链接，解析信息后下载到本地目录。

## 当前状态

已完成项目骨架、核心交互和本机运行时接入，并已生成 Windows x64 安装包。应用只在本机运行；解析远程视频链接时仍需要访问对应原站。

## 本地开发

```powershell
npm install
npm run dev
```

桌面开发需要 Rust/Cargo 和 Visual Studio C++ Build Tools：

```powershell
npm run tauri:dev
```

Windows 打包需要先在带有 MSVC 环境的终端中执行：

```powershell
call C:\BuildTools\Common7\Tools\VsDevCmd.bat -arch=x64 -host_arch=x64
npm run tauri:build
```

## 运行时

项目当前已放入 `yt-dlp.exe`、`ffmpeg.exe`、`ffprobe.exe`；换机或更新版本时，按 [tools/README.md](tools/README.md) 替换即可。

仓库不会提交这些第三方运行时二进制、构建目录或本地视频。首次开发或打包前，请按 [tools/README.md](tools/README.md) 准备运行时文件；它们各自的许可证和更新规则也与本项目分开。

## 设计约束

- 不启动 HTTP 服务，不对外监听端口。
- 提取到的目标只接受 `http`/`https` URL。
- 支持直接粘贴带说明文字的分享内容，自动提取其中的第一个 `http`/`https` 链接。
- 单条视频下载，默认不展开播放列表。
- 所有子进程都用参数数组启动，不拼接 shell 命令。
- 首版不开放任意 yt-dlp 参数，避免路径穿越和命令注入。

## 许可证

本项目代码采用 [MIT License](LICENSE) 开源免费使用。`yt-dlp`、FFmpeg、FFprobe 和 Deno 属于独立的第三方项目，使用或分发它们时请遵守各自许可证；详见 [tools/README.md](tools/README.md)。
