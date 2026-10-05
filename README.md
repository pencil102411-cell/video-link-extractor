# Video Link Extractor

一个只在本机运行的桌面工具：粘贴视频链接，解析信息后下载到本地目录。

## 当前状态

已完成项目骨架、核心交互和本机运行时接入；前端检查通过。最终 Windows 安装包仍需要本机 Visual Studio C++ Build Tools 的 `link.exe`。

## 本地开发

```powershell
npm install
npm run dev
```

桌面开发需要 Rust/Cargo 和 Windows C++ Build Tools：

```powershell
npm run tauri:dev
```

## 运行时

项目当前已放入 `yt-dlp.exe`、`ffmpeg.exe`、`ffprobe.exe`；换机或更新版本时，按 [tools/README.md](tools/README.md) 替换即可。

## 设计约束

- 不启动 HTTP 服务，不对外监听端口。
- 只接受 `http`/`https` URL。
- 单条视频下载，默认不展开播放列表。
- 所有子进程都用参数数组启动，不拼接 shell 命令。
- 首版不开放任意 yt-dlp 参数，避免路径穿越和命令注入。
