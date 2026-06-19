use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

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

/// Lists sibling directories of `folder` (other subfolders of its parent),
/// naturally sorted, so Up/Down can step between adjacent shoot/album folders.
#[tauri::command]
fn sibling_folders(folder: String) -> Vec<String> {
    let dir = Path::new(&folder);
    let parent = match dir.parent() {
        Some(p) => p,
        None => return Vec::new(),
    };
    let read_dir = match std::fs::read_dir(parent) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut folders: Vec<String> = read_dir
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    folders.sort_by(|a, b| {
        let an = Path::new(a).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let bn = Path::new(b).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        natural_key(&an).cmp(&natural_key(&bn))
    });
    folders
}

#[derive(Serialize, Deserialize, Clone)]
struct TrashEntry {
    id: String,
    original_path: String,
    stored_path: String,
    deleted_at: i64,
}

fn app_support_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn trash_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_support_dir(app)?.join("trash");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn trash_manifest_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_support_dir(app)?.join("trash_manifest.json"))
}

fn read_trash_manifest(app: &AppHandle) -> Result<Vec<TrashEntry>, String> {
    let path = trash_manifest_path(app)?;
    match std::fs::read_to_string(&path) {
        Ok(s) => Ok(serde_json::from_str(&s).unwrap_or_default()),
        Err(_) => Ok(Vec::new()),
    }
}

fn write_trash_manifest(app: &AppHandle, entries: &[TrashEntry]) -> Result<(), String> {
    let path = trash_manifest_path(app)?;
    let json = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

fn move_file(from: &Path, to: &Path) -> Result<(), String> {
    if std::fs::rename(from, to).is_ok() {
        return Ok(());
    }
    // Rename fails across filesystems/volumes; fall back to copy + remove.
    std::fs::copy(from, to).map_err(|e| e.to_string())?;
    std::fs::remove_file(from).map_err(|e| e.to_string())
}

/// Moves files into an app-managed trash folder (instead of the OS trash) so
/// the app can reliably restore them later via `restore_paths`.
#[tauri::command]
fn soft_delete(app: AppHandle, paths: Vec<String>) -> Result<Vec<TrashEntry>, String> {
    let dir = trash_dir(&app)?;
    let mut manifest = read_trash_manifest(&app)?;
    let mut created = Vec::new();

    for original in &paths {
        let src = Path::new(original);
        let name = src
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let id = format!("{}-{}", now_millis(), name);
        let dest = dir.join(&id);
        move_file(src, &dest)?;

        let entry = TrashEntry {
            id: id.clone(),
            original_path: original.clone(),
            stored_path: dest.to_string_lossy().to_string(),
            deleted_at: now_millis(),
        };
        manifest.push(entry.clone());
        created.push(entry);
    }

    write_trash_manifest(&app, &manifest)?;
    Ok(created)
}

/// Restores previously soft-deleted files back to their original locations.
#[tauri::command]
fn restore_paths(app: AppHandle, original_paths: Vec<String>) -> Result<(), String> {
    let mut manifest = read_trash_manifest(&app)?;
    let targets: std::collections::HashSet<&str> =
        original_paths.iter().map(|s| s.as_str()).collect();

    let mut remaining = Vec::new();
    for entry in manifest.drain(..) {
        if targets.contains(entry.original_path.as_str()) {
            let stored = Path::new(&entry.stored_path);
            let original = Path::new(&entry.original_path);
            if let Some(parent) = original.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            move_file(stored, original)?;
        } else {
            remaining.push(entry);
        }
    }
    manifest = remaining;
    write_trash_manifest(&app, &manifest)
}

/// Permanently purges everything in the app-managed trash (used by "Clear
/// History & Cache" — after this, undo is no longer possible for past deletes).
#[tauri::command]
fn clear_trash(app: AppHandle) -> Result<(), String> {
    let dir = trash_dir(&app)?;
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    write_trash_manifest(&app, &[])
}

#[derive(Serialize, Deserialize, Clone)]
struct HistoryEntry {
    folder: String,
    last_item: Option<String>,
    opened_at: i64,
}

const MAX_HISTORY: usize = 20;

fn history_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_support_dir(app)?.join("history.json"))
}

/// Returns recently opened folders, most-recent first, so the app can resume
/// the last session and offer a "Recent" list.
#[tauri::command]
fn get_history(app: AppHandle) -> Result<Vec<HistoryEntry>, String> {
    let path = history_path(&app)?;
    match std::fs::read_to_string(&path) {
        Ok(s) => Ok(serde_json::from_str(&s).unwrap_or_default()),
        Err(_) => Ok(Vec::new()),
    }
}

/// Upserts a folder's history entry (and its last-viewed item) and moves it
/// to the front, capped at `MAX_HISTORY` entries.
#[tauri::command]
fn record_history(
    app: AppHandle,
    folder: String,
    last_item: Option<String>,
) -> Result<Vec<HistoryEntry>, String> {
    let path = history_path(&app)?;
    let mut entries: Vec<HistoryEntry> = match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    entries.retain(|e| e.folder != folder);
    entries.insert(
        0,
        HistoryEntry {
            folder,
            last_item,
            opened_at: now_millis(),
        },
    );
    entries.truncate(MAX_HISTORY);

    let json = serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(entries)
}

#[tauri::command]
fn clear_history(app: AppHandle) -> Result<(), String> {
    let path = history_path(&app)?;
    std::fs::write(path, "[]").map_err(|e| e.to_string())
}

#[derive(Serialize, Clone)]
struct DuplicateGroup {
    hash: String,
    paths: Vec<String>,
}

/// Groups files in `folder` that share identical content (by size, then
/// SHA-256), so the UI can offer to clean up exact duplicates.
#[tauri::command]
fn find_duplicates(folder: String) -> Result<Vec<DuplicateGroup>, String> {
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;

    let dir = Path::new(&folder);
    let read_dir = std::fs::read_dir(dir).map_err(|e| e.to_string())?;

    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for item in read_dir.flatten() {
        let path = item.path();
        if !path.is_file() {
            continue;
        }
        if let Ok(meta) = item.metadata() {
            by_size.entry(meta.len()).or_default().push(path);
        }
    }

    let mut by_hash: HashMap<String, Vec<String>> = HashMap::new();
    for (_, paths) in by_size.into_iter().filter(|(_, p)| p.len() > 1) {
        for path in paths {
            let bytes = match std::fs::read(&path) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let hash = format!("{:x}", hasher.finalize());
            by_hash
                .entry(hash)
                .or_default()
                .push(path.to_string_lossy().to_string());
        }
    }

    Ok(by_hash
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(hash, paths)| DuplicateGroup { hash, paths })
        .collect())
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
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(InitialPath(Mutex::new(first_existing_path_arg())))
        .invoke_handler(tauri::generate_handler![
            scan_folder,
            parent_dir,
            take_initial_path,
            sibling_folders,
            soft_delete,
            restore_paths,
            clear_trash,
            find_duplicates,
            get_history,
            record_history,
            clear_history
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            let _ = (&app, &event);
            // `RunEvent::Opened` (Finder/Dock "Open With") only exists on macOS/iOS.
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            if let tauri::RunEvent::Opened { urls } = event {
                if let Some(path) = urls.iter().find_map(|u| url_to_path(u.as_str())) {
                    deliver_path(app, path);
                }
            }
        });
}
