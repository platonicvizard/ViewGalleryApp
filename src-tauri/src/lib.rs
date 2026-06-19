use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};

const IMAGE_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "tif", "tiff", "avif", "heic", "heif", "svg",
];
const VIDEO_EXTS: &[&str] = &[
    "mp4", "mov", "webm", "mkv", "avi", "m4v", "wmv", "flv", "mpg", "mpeg",
];

#[derive(Serialize, Clone)]
struct MediaEntry {
    path: String,
    name: String,
    kind: &'static str,
    ext: String,
    size: u64,
    modified: i64,
}

fn media_kind(ext: &str) -> Option<&'static str> {
    let ext = ext.to_lowercase();
    if IMAGE_EXTS.contains(&ext.as_str()) {
        Some("image")
    } else if VIDEO_EXTS.contains(&ext.as_str()) {
        Some("video")
    } else {
        None
    }
}

/// Splits a filename into alternating text/number chunks so names like
/// "img2.jpg" sort before "img10.jpg" instead of using plain lexical order.
fn natural_key(name: &str) -> Vec<(String, u64)> {
    let mut key = Vec::new();
    let mut chars = name.chars().peekable();
    while chars.peek().is_some() {
        let is_digit = chars.peek().unwrap().is_ascii_digit();
        let mut chunk = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() == is_digit {
                chunk.push(c);
                chars.next();
            } else {
                break;
            }
        }
        if is_digit {
            key.push((String::new(), chunk.parse().unwrap_or(0)));
        } else {
            key.push((chunk.to_lowercase(), 0));
        }
    }
    key
}

#[tauri::command]
fn scan_folder(folder: String) -> Result<Vec<MediaEntry>, String> {
    let dir = Path::new(&folder);
    let read_dir = std::fs::read_dir(dir).map_err(|e| e.to_string())?;

    let mut entries: Vec<MediaEntry> = Vec::new();
    for item in read_dir.flatten() {
        let path = item.path();
        if !path.is_file() {
            continue;
        }
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_string(),
            None => continue,
        };
        let kind = match media_kind(&ext) {
            Some(k) => k,
            None => continue,
        };
        let meta = match item.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        entries.push(MediaEntry {
            path: path.to_string_lossy().to_string(),
            name,
            kind,
            ext: ext.to_lowercase(),
            size: meta.len(),
            modified,
        });
    }

    entries.sort_by(|a, b| natural_key(&a.name).cmp(&natural_key(&b.name)));
    Ok(entries)
}

#[tauri::command]
fn parent_dir(path: String) -> Option<String> {
    Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
}

#[derive(Default)]
struct InitialPath(Mutex<Option<String>>);

#[tauri::command]
fn take_initial_path(state: State<InitialPath>) -> Option<String> {
    state.0.lock().unwrap().take()
}

fn first_existing_path_arg() -> Option<String> {
    std::env::args().skip(1).find_map(|arg| {
        let p = PathBuf::from(&arg);
        if p.exists() {
            Some(p.to_string_lossy().to_string())
        } else {
            None
        }
    })
}

fn url_to_path(url: &str) -> Option<String> {
    url.strip_prefix("file://").map(urlencoding_decode)
}

fn urlencoding_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn deliver_path(app: &tauri::AppHandle, path: String) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_focus();
        let _ = app.emit("open-path", path.clone());
    }
    if let Some(state) = app.try_state::<InitialPath>() {
        *state.0.lock().unwrap() = Some(path);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(path) = argv.iter().skip(1).find(|a| Path::new(a).exists()) {
                deliver_path(app, path.clone());
            }
        }));
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(InitialPath(Mutex::new(first_existing_path_arg())))
        .invoke_handler(tauri::generate_handler![
            scan_folder,
            parent_dir,
            take_initial_path
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Opened { urls } = event {
                if let Some(path) = urls.iter().find_map(|u| url_to_path(u.as_str())) {
                    deliver_path(app, path);
                }
            }
        });
}
