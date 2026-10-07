# Windows 构建前置条件

当前源码和前端构建已完成，Windows Tauri 打包还需要微软 C++ 工具链。安装官方 Visual Studio Build Tools 时选择：

- **Desktop development with C++**
- MSVC x64/x86 build tools
- Windows 10 或 Windows 11 SDK

安装器如果一直停留在“正在进行准备”，不代表组件已安装完成。等待安装器进入工作负载列表后，确认勾选 **Desktop development with C++**；命令行对应的工作负载 ID 是 `Microsoft.VisualStudio.Workload.VCTools`。

安装完成后，重新打开 PowerShell，在项目根目录执行：

```powershell
cd "$env:USERPROFILE\Desktop\video-link-extractor"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
npm install
npm run check
npm run tauri:build
```

截图中 `Missing script: "tauri:build"` 的原因是当前目录仍是用户主目录，不是项目根目录。

本机验证标准是 `where.exe link.exe` 能找到 MSVC 的 `link.exe`。如果仍找不到，可从“x64 Native Tools Command Prompt for VS”执行构建，或先运行 Visual Studio Installer 的“修复”。

官方入口：<https://visualstudio.microsoft.com/visual-cpp-build-tools/>
