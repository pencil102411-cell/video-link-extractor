use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use url::Url;
use uuid::Uuid;

#[derive(Clone, Default)]
struct AppState {
    tasks: Arc<Mutex<HashMap<String, Arc<Mutex<Child>>>>>,
    cancelled: Arc<Mutex<HashSet<String>>>,
    active: Arc<Mutex<bool>>,
    completed_paths: Arc<Mutex<HashSet<PathBuf>>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DependencyStatus {
    name: String,
    available: bool,
    version: Option<String>,
    path: Option<String>,
    required: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FormatSummary {
    height: Option<u64>,
    ext: Option<String>,
    fps: Option<f64>,
    video_codec: Option<String>,
    audio_codec: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VideoInfo {
    title: String,
    duration: Option<f64>,
    thumbnail: Option<String>,
    extractor: Option<String>,
    uploader: Option<String>,
    width: Option<u64>,
    height: Option<u64>,
    formats: Vec<FormatSummary>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadRequest {
    url: String,
    output_dir: String,
    profile: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadEvent {
    task_id: String,
    state: String,
    percent: Option<f64>,
    speed: Option<String>,
    eta: Option<String>,
    filepath: Option<String>,
    message: Option<String>,
}

fn tools_dir(app: &AppHandle) -> PathBuf {
    let dev_tools = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tools");
    if dev_tools.exists() {
        return dev_tools;
    }

    app.path()
        .resource_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("tools")
}

fn tool_path(app: &AppHandle, name: &str) -> PathBuf {
    let candidate = tools_dir(app).join(name);
    if candidate.exists() {
        candidate
    } else {
        PathBuf::from(name)
    }
}

fn configure_hidden_window(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
}

fn run_version(path: &Path) -> Option<String> {
    let mut command = Command::new(path);
    command.arg("--version");
    configure_hidden_window(&mut command);
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let version = stdout
        .lines()
        .chain(stderr.lines())
        .find(|line| !line.trim().is_empty())?;
    Some(version.trim().to_string())
}

fn inspect_tool(app: &AppHandle, name: &str, required: bool) -> DependencyStatus {
    let candidate = tool_path(app, name);
    let version = run_version(&candidate);
    let path = if version.is_some() {
        if candidate.exists() {
            Some(candidate.to_string_lossy().to_string())
        } else {
            Some(format!("PATH: {name}"))
        }
    } else {
        None
    };

    DependencyStatus {
        name: name.trim_end_matches(".exe").to_string(),
        available: version.is_some(),
        version,
        path,
        required,
    }
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(value) => {
            let octets = value.octets();
            octets[0] == 0
                || octets[0] == 10
                || octets[0] == 127
                || (octets[0] == 169 && octets[1] == 254)
                || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                || (octets[0] == 192 && octets[1] == 168)
        }
        IpAddr::V6(value) => {
            value.is_loopback()
                || value.is_unique_local()
                || value.is_unicast_link_local()
                || value
                    .to_ipv4()
                    .is_some_and(|mapped| is_blocked_ip(IpAddr::V4(mapped)))
        }
    }
}

fn validate_url(raw: &str) -> Result<(), String> {
    let trimmed = raw.trim();
    let parsed = Url::parse(trimmed).map_err(|_| "链接格式不正确".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("只支持 http:// 或 https:// 视频链接".to_string());
    }

    let host = parsed
        .host_str()
        .unwrap_or_default()
        .trim_end_matches('.')
        .to_ascii_lowercase();

    if host.is_empty() || host == "localhost" || host.ends_with(".local") {
        return Err("不允许访问本机或局域网地址".to_string());
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(ip) {
            return Err("不允许访问本机或局域网地址".to_string());
        }
    }
    if let Some(port) = parsed.port_or_known_default() {
        if let Ok(addresses) = (host.as_str(), port).to_socket_addrs() {
            if addresses
                .into_iter()
                .any(|address| is_blocked_ip(address.ip()))
            {
                return Err("链接解析到了本机或局域网地址".to_string());
            }
        }
    }
    Ok(())
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn json_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn json_f64(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn format_summaries(info: &Value) -> Vec<FormatSummary> {
    let mut seen = HashSet::new();
    let mut formats = info
        .get("formats")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|format| {
            let video_codec = json_string(format, "vcodec");
            if video_codec.as_deref().unwrap_or("none") == "none" {
                return None;
            }
            let summary = FormatSummary {
                height: json_u64(format, "height"),
                ext: json_string(format, "ext"),
                fps: json_f64(format, "fps"),
                video_codec,
                audio_codec: json_string(format, "acodec"),
            };
            let key = format!(
                "{:?}|{:?}|{:?}|{:?}",
                summary.height, summary.ext, summary.fps, summary.audio_codec
            );
            if seen.insert(key) {
                Some(summary)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    formats.sort_by(|left, right| right.height.cmp(&left.height));
    formats.truncate(12);
    formats
}

#[tauri::command]
fn default_output_dir() -> Result<String, String> {
    let user_profile =
        std::env::var_os("USERPROFILE").ok_or_else(|| "无法读取用户目录".to_string())?;
    let path = PathBuf::from(user_profile).join("Downloads");
    fs::create_dir_all(&path).map_err(|error| format!("创建下载目录失败：{error}"))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn check_dependencies(app: AppHandle) -> Vec<DependencyStatus> {
    vec![
        inspect_tool(&app, "yt-dlp.exe", true),
        inspect_tool(&app, "ffmpeg.exe", true),
        inspect_tool(&app, "ffprobe.exe", true),
        inspect_tool(&app, "deno.exe", false),
    ]
}

fn require_tool(app: &AppHandle, name: &str) -> Result<(), String> {
    let path = tool_path(app, name);
    if run_version(&path).is_some() {
        Ok(())
    } else {
        Err(format!(
            "缺少运行时：{}，请先放入 tools 目录或加入系统 PATH",
            name.trim_end_matches(".exe")
        ))
    }
}

fn configure_js_runtime(app: &AppHandle, command: &mut Command) {
    let deno = tool_path(app, "deno.exe");
    if deno.exists() {
        command
            .arg("--js-runtimes")
            .arg(format!("deno:{}", deno.display()));
    }
}

#[tauri::command]
fn inspect_url(app: AppHandle, url: String) -> Result<VideoInfo, String> {
    validate_url(&url)?;
    let executable = tool_path(&app, "yt-dlp.exe");
    let mut command = Command::new(executable);
    configure_js_runtime(&app, &mut command);
    command.args([
        "--dump-single-json",
        "--skip-download",
        "--no-playlist",
        "--ignore-config",
        "--no-warnings",
        "--",
        url.trim(),
    ]);
    configure_hidden_window(&mut command);
    let output = command
        .output()
        .map_err(|error| format!("启动 yt-dlp 失败：{error}"))?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if message.is_empty() {
            "链接解析失败，请确认运行时文件和视频链接可用".to_string()
        } else {
            message
        });
    }

    let info: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("解析 yt-dlp 返回值失败：{error}"))?;
    Ok(VideoInfo {
        title: json_string(&info, "title").unwrap_or_else(|| "未命名视频".to_string()),
        duration: json_f64(&info, "duration"),
        thumbnail: json_string(&info, "thumbnail"),
        extractor: json_string(&info, "extractor_key").or_else(|| json_string(&info, "extractor")),
        uploader: json_string(&info, "uploader").or_else(|| json_string(&info, "channel")),
        width: json_u64(&info, "width"),
        height: json_u64(&info, "height"),
        formats: format_summaries(&info),
    })
}

fn format_selector(profile: &str) -> &'static str {
    match profile {
        "1080" => "bv*[height<=1080]+ba/b[height<=1080]/b",
        "720" => "bv*[height<=720]+ba/b[height<=720]/b",
        _ => "bv*+ba/b",
    }
}

fn emit_event(app: &AppHandle, event: DownloadEvent) {
    let _ = app.emit("download://progress", event);
}

fn parse_percent(value: &str) -> Option<f64> {
    value
        .trim()
        .trim_end_matches('%')
        .trim()
        .parse::<f64>()
        .ok()
}

fn parse_progress_line(
    line: &str,
) -> Option<(String, Option<f64>, Option<String>, Option<String>)> {
    let (event_state, payload) = if let Some(value) = line.strip_prefix("download:") {
        ("downloading", value)
    } else if let Some(value) = line.strip_prefix("postprocess:") {
        ("processing", value)
    } else {
        return None;
    };
    let mut percent = None;
    let mut speed = None;
    let mut eta = None;
    for item in payload.split('|') {
        let Some((key, value)) = item.split_once('=') else {
            continue;
        };
        match key {
            "percent" => percent = parse_percent(value),
            "speed" => speed = Some(value.trim().to_string()),
            "eta" => eta = Some(value.trim().to_string()),
            _ => {}
        }
    }
    Some((event_state.to_string(), percent, speed, eta))
}

#[tauri::command]
fn start_download(
    app: AppHandle,
    state: State<AppState>,
    request: DownloadRequest,
) -> Result<String, String> {
    validate_url(&request.url)?;
    require_tool(&app, "yt-dlp.exe")?;
    require_tool(&app, "ffmpeg.exe")?;
    require_tool(&app, "ffprobe.exe")?;
    {
        let mut active = state
            .active
            .lock()
            .map_err(|_| "任务状态锁定失败".to_string())?;
        if *active {
            return Err("已有下载任务正在进行，请等待完成或取消后再试".to_string());
        }
        *active = true;
    }
    let output_dir = PathBuf::from(request.output_dir.trim());
    if output_dir.as_os_str().is_empty() {
        if let Ok(mut active) = state.active.lock() {
            *active = false;
        }
        return Err("请选择下载目录".to_string());
    }
    if let Err(error) = fs::create_dir_all(&output_dir) {
        if let Ok(mut active) = state.active.lock() {
            *active = false;
        }
        return Err(format!("创建下载目录失败：{error}"));
    }

    let task_id = Uuid::new_v4().to_string();
    let executable = tool_path(&app, "yt-dlp.exe");
    let ffmpeg_dir = tools_dir(&app);
    let progress_template = "download:status=%(progress.status)s|percent=%(progress._percent_str)s|speed=%(progress._speed_str)s|eta=%(progress._eta_str)s";
    let mut command = Command::new(executable);
    command.args([
        "--ignore-config",
        "--no-playlist",
        "--newline",
        "--quiet",
        "--no-warnings",
        "--continue",
        "--part",
        "--progress-template",
        progress_template,
        "--progress-template",
        "postprocess:status=%(progress.status)s|percent=%(progress._percent_str)s|speed=%(progress._speed_str)s|eta=%(progress._eta_str)s",
        "--print",
        "after_move:final=%(filepath)s",
        "--format",
        format_selector(&request.profile),
        "--merge-output-format",
        "mp4",
        "--output",
        "%(title).200B [%(id)s].%(ext)s",
    ]);
    command
        .arg("--paths")
        .arg(format!("home={}", output_dir.display()));
    if ffmpeg_dir.join("ffmpeg.exe").exists() {
        command.arg("--ffmpeg-location").arg(&ffmpeg_dir);
    }
    configure_js_runtime(&app, &mut command);
    command.args(["--", request.url.trim()]);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    configure_hidden_window(&mut command);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            if let Ok(mut active) = state.active.lock() {
                *active = false;
            }
            return Err(format!("启动 yt-dlp 失败：{error}"));
        }
    };
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let child_handle = Arc::new(Mutex::new(child));
    state
        .tasks
        .lock()
        .map_err(|_| "任务状态锁定失败".to_string())?
        .insert(task_id.clone(), child_handle.clone());

    let task_state = state.inner().clone();
    let task_id_for_thread = task_id.clone();
    let app_for_thread = app.clone();
    thread::spawn(move || {
        emit_event(
            &app_for_thread,
            DownloadEvent {
                task_id: task_id_for_thread.clone(),
                state: "downloading".to_string(),
                percent: Some(0.0),
                speed: None,
                eta: None,
                filepath: None,
                message: None,
            },
        );

        let stderr_thread = thread::spawn(move || {
            stderr
                .map(|pipe| {
                    BufReader::new(pipe)
                        .lines()
                        .filter_map(Result::ok)
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default()
        });

        let mut filepath = None;
        if let Some(pipe) = stdout {
            for line in BufReader::new(pipe).lines().filter_map(Result::ok) {
                if let Some((event_state, percent, speed, eta)) = parse_progress_line(&line) {
                    emit_event(
                        &app_for_thread,
                        DownloadEvent {
                            task_id: task_id_for_thread.clone(),
                            state: event_state,
                            percent,
                            speed,
                            eta,
                            filepath: None,
                            message: None,
                        },
                    );
                } else if let Some(value) = line.strip_prefix("final=") {
                    filepath = Some(value.trim().to_string());
                }
            }
        }

        let status = loop {
            let result = child_handle
                .lock()
                .ok()
                .and_then(|mut child| child.try_wait().ok().flatten());
            if let Some(status) = result {
                break status;
            }
            thread::sleep(Duration::from_millis(100));
        };
        let stderr_message = stderr_thread.join().unwrap_or_default();
        let cancelled = task_state
            .cancelled
            .lock()
            .map(|items| items.contains(&task_id_for_thread))
            .unwrap_or(false);
        let (state_name, message) = if cancelled {
            ("cancelled".to_string(), Some("任务已取消".to_string()))
        } else if status.success() {
            ("completed".to_string(), None)
        } else {
            let fallback = "yt-dlp 下载失败".to_string();
            (
                "failed".to_string(),
                Some(if stderr_message.trim().is_empty() {
                    fallback
                } else {
                    stderr_message
                }),
            )
        };
        emit_event(
            &app_for_thread,
            DownloadEvent {
                task_id: task_id_for_thread.clone(),
                state: state_name,
                percent: if status.success() { Some(100.0) } else { None },
                speed: None,
                eta: None,
                filepath: filepath.clone(),
                message,
            },
        );
        if let Ok(mut tasks) = task_state.tasks.lock() {
            tasks.remove(&task_id_for_thread);
        }
        if status.success() {
            if let Some(path) = filepath
                .as_ref()
                .and_then(|value| fs::canonicalize(value).ok())
            {
                if let Ok(mut completed) = task_state.completed_paths.lock() {
                    completed.insert(path);
                }
            }
        }
        if let Ok(mut active) = task_state.active.lock() {
            *active = false;
        }
        if let Ok(mut cancelled_items) = task_state.cancelled.lock() {
            cancelled_items.remove(&task_id_for_thread);
        }
    });

    Ok(task_id)
}

#[tauri::command]
fn cancel_download(state: State<AppState>, task_id: String) -> Result<(), String> {
    state
        .cancelled
        .lock()
        .map_err(|_| "任务状态锁定失败".to_string())?
        .insert(task_id.clone());
    let task = state
        .tasks
        .lock()
        .map_err(|_| "任务状态锁定失败".to_string())?
        .get(&task_id)
        .cloned()
        .ok_or_else(|| "任务不存在或已结束".to_string())?;
    let mut child = task.lock().map_err(|_| "任务锁定失败".to_string())?;
    if child
        .try_wait()
        .map_err(|error| format!("读取任务状态失败：{error}"))?
        .is_some()
    {
        return Ok(());
    }
    #[cfg(windows)]
    {
        let pid = child.id().to_string();
        let mut command = Command::new("taskkill");
        command.args(["/PID", &pid, "/T", "/F"]);
        configure_hidden_window(&mut command);
        command
            .status()
            .map_err(|error| format!("取消任务失败：{error}"))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        child
            .kill()
            .map_err(|error| format!("取消任务失败：{error}"))
    }
}

#[tauri::command]
fn open_path(state: State<AppState>, path: String) -> Result<(), String> {
    let target = fs::canonicalize(path.trim()).map_err(|_| "文件或目录不存在".to_string())?;
    let allowed = state
        .completed_paths
        .lock()
        .map_err(|_| "任务状态锁定失败".to_string())?
        .contains(&target);
    if !allowed {
        return Err("只能打开本次任务已完成的文件".to_string());
    }
    if !target.exists() {
        return Err("文件或目录不存在".to_string());
    }
    #[cfg(windows)]
    {
        let mut command = Command::new("explorer.exe");
        if target.is_dir() {
            command.arg(&target);
        } else {
            command.arg(format!("/select,{}", target.display()));
        }
        configure_hidden_window(&mut command);
        command
            .spawn()
            .map_err(|error| format!("打开路径失败：{error}"))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = target;
        Err("当前版本只支持 Windows".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            default_output_dir,
            check_dependencies,
            inspect_url,
            start_download,
            cancel_download,
            open_path
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_public_http_and_https_urls() {
        assert!(validate_url("https://example.com/video?id=1").is_ok());
        assert!(validate_url("http://media.example.org/file.mp4").is_ok());
    }

    #[test]
    fn rejects_local_and_non_http_urls() {
        for url in [
            "file:///C:/video.mp4",
            "ftp://example.com/video.mp4",
            "http://localhost/video",
            "http://127.0.0.1/video",
            "http://192.168.1.10/video",
            "http://[::1]/video",
            "http://[fe80::1]/video",
            "http://user:pass@127.0.0.1/video",
            "http://[::ffff:127.0.0.1]/video",
        ] {
            assert!(validate_url(url).is_err(), "should reject {url}");
        }
    }

    #[test]
    fn profile_selector_never_accepts_arbitrary_format_expression() {
        assert_eq!(
            format_selector("1080"),
            "bv*[height<=1080]+ba/b[height<=1080]/b"
        );
        assert_eq!(
            format_selector("720"),
            "bv*[height<=720]+ba/b[height<=720]/b"
        );
        assert_eq!(format_selector("unexpected"), "bv*+ba/b");
    }

    #[test]
    fn parses_progress_events() {
        let parsed = parse_progress_line(
            "download:status=downloading|percent=42.5%|speed=2.1MiB/s|eta=00:03",
        )
        .expect("progress line");
        assert_eq!(parsed.0, "downloading");
        assert_eq!(parsed.1, Some(42.5));
        assert_eq!(parsed.2.as_deref(), Some("2.1MiB/s"));
        assert_eq!(parsed.3.as_deref(), Some("00:03"));
        assert!(parse_progress_line("ordinary log line").is_none());
    }
}
