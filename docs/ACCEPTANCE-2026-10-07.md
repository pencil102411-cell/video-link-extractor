# 验收记录（2026-10-07）

## 测试链接

`https://xhslink.cn/o/2KqnhGIdSrI`

## 混合分享文本测试

输入形态：`黑蝙蝠中队，已准备起飞！2002年的旋律一响，DNA动了... https://xhslink.cn/o/2KqnhGIdSrI，先复制一下，再到浏览器打开`

结果：成功提取首个链接，未把前后说明文字传给 URL 校验或 yt-dlp。

## 结果

- 短链跳转到小红书登录包装页后，成功提取 `redirectPath` 并解析真实内容地址。
- 使用 `yt-dlp 2026.08.19` 下载成功，退出码为 `0`。
- 输出文件：`work/acceptance-xhs-download2/黑蝙蝠中队，已准备起飞！ [694f71aa000000001e02c05a].mp4`
- 文件大小：28,829,637 bytes。
- `ffprobe` 校验：MP4 容器、H.264 视频、AAC 音频、1280×720、254.966667 秒。

## 修复项

- 支持把“说明文字 + 视频链接 + 后续说明”作为输入，自动提取首个 HTTP/HTTPS 链接。
- FFmpeg/FFprobe 版本探测同时兼容 `--version` 和 `-version`。
- Windows 下载目录参数改为 yt-dlp 要求的 `home:<目录>` 格式。
- 安装版运行时兼容 `resource_dir`、exe 同级和旧版 `_up_/tools` 路径。

## 自动检查

- `npm run check`：通过。
- `cargo test`：9 项通过。
- `npm run tauri:build`：0.1.1 版 MSI 和 NSIS 均构建成功。
