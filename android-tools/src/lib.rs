use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;
use tokio::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

const ADB_EXE: &[u8] = include_bytes!("../scrcpy_files/adb.exe");
const ADB_WIN_API_DLL: &[u8] = include_bytes!("../scrcpy_files/AdbWinApi.dll");
const ADB_WIN_USB_DLL: &[u8] = include_bytes!("../scrcpy_files/AdbWinUsbApi.dll");
const SCRCPY_EXE: &[u8] = include_bytes!("../scrcpy_files/scrcpy.exe");
const SCRCPY_SERVER: &[u8] = include_bytes!("../scrcpy_files/scrcpy-server");
const SDL2_DLL: &[u8] = include_bytes!("../scrcpy_files/SDL2.dll");
const AVCODEC_DLL: &[u8] = include_bytes!("../scrcpy_files/avcodec-61.dll");
const AVFORMAT_DLL: &[u8] = include_bytes!("../scrcpy_files/avformat-61.dll");
const AVUTIL_DLL: &[u8] = include_bytes!("../scrcpy_files/avutil-59.dll");
const SWRESAMPLE_DLL: &[u8] = include_bytes!("../scrcpy_files/swresample-5.dll");
const LIBUSB_DLL: &[u8] = include_bytes!("../scrcpy_files/libusb-1.0.dll");

fn get_tools_cache_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join(".tools_cache")
}

fn ensure_tools_extracted() -> Result<PathBuf, String> {
    let cache_dir = get_tools_cache_dir();
    let adb_exe_path = cache_dir.join("adb.exe");

    if adb_exe_path.exists() {
        return Ok(cache_dir);
    }

    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("创建缓存目录失败: {}", e))?;

    let files: Vec<(&str, &[u8])> = vec![
        ("adb.exe", ADB_EXE),
        ("AdbWinApi.dll", ADB_WIN_API_DLL),
        ("AdbWinUsbApi.dll", ADB_WIN_USB_DLL),
        ("scrcpy.exe", SCRCPY_EXE),
        ("scrcpy-server", SCRCPY_SERVER),
        ("SDL2.dll", SDL2_DLL),
        ("avcodec-61.dll", AVCODEC_DLL),
        ("avformat-61.dll", AVFORMAT_DLL),
        ("avutil-59.dll", AVUTIL_DLL),
        ("swresample-5.dll", SWRESAMPLE_DLL),
        ("libusb-1.0.dll", LIBUSB_DLL),
    ];

    for (name, data) in files {
        let file_path = cache_dir.join(name);
        std::fs::write(&file_path, data)
            .map_err(|e| format!("写入{}失败: {}", name, e))?;
    }

    Ok(cache_dir)
}

#[derive(Serialize, Deserialize, Clone)]
struct Device {
    device_id: String,
    status: String,
    model: String,
}

#[derive(Serialize, Deserialize)]
struct CommandResult {
    success: bool,
    output: String,
}

struct AppState {
    adb_path: Mutex<Option<String>>,
}

#[tauri::command]
async fn find_adb(state: State<'_, AppState>) -> Result<Option<String>, ()> {
    {
        let path = state.adb_path.lock().unwrap();
        if path.is_some() {
            return Ok(path.clone());
        }
    }

    match ensure_tools_extracted() {
        Ok(cache_dir) => {
            let adb_path = cache_dir.join("adb.exe");
            if adb_path.exists() {
                let path = adb_path.to_string_lossy().to_string();
                *state.adb_path.lock().unwrap() = Some(path.clone());
                return Ok(Some(path));
            }
        }
        Err(_) => {}
    }

    Ok(None)
}

#[tauri::command]
async fn set_adb_path(path: String, state: State<'_, AppState>) -> Result<bool, ()> {
    if Path::new(&path).exists() {
        *state.adb_path.lock().unwrap() = Some(path);
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
async fn get_adb_path(state: State<'_, AppState>) -> Result<Option<String>, ()> {
    Ok(state.adb_path.lock().unwrap().clone())
}

async fn run_adb_command_async(args: &[&str], state: &State<'_, AppState>) -> CommandResult {
    let adb_path = match state.adb_path.lock().unwrap().clone() {
        Some(path) => path,
        None => return CommandResult {
            success: false,
            output: "ADB not found".to_string(),
        },
    };

    #[cfg(windows)]
    let output = {
        let mut cmd = std::process::Command::new(&adb_path);
        cmd.args(args);
        cmd.creation_flags(CREATE_NO_WINDOW);
        let result = tokio::task::spawn_blocking(move || cmd.output()).await;
        match result {
            Ok(Ok(o)) => Ok(o),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Task failed")),
        }
    };

    #[cfg(not(windows))]
    let output = Command::new(&adb_path).args(args).output().await;

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            let mut output = stdout.to_string();
            if !stderr.is_empty() {
                output.push('\n');
                output.push_str(&stderr);
            }
            CommandResult {
                success: result.status.success(),
                output: output.trim().to_string(),
            }
        }
        Err(e) => CommandResult {
            success: false,
            output: format!("Error: {}", e),
        },
    }
}

#[tauri::command]
async fn connect_device(ip: String, port: u16, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    let address = format!("{}:{}", ip, port);
    Ok(run_adb_command_async(&["connect", &address], &state).await)
}

#[tauri::command]
async fn disconnect_device(ip: Option<String>, port: Option<u16>, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    if let Some(ip_addr) = ip {
        let port_num = port.unwrap_or(5555);
        let address = format!("{}:{}", ip_addr, port_num);
        Ok(run_adb_command_async(&["disconnect", &address], &state).await)
    } else {
        Ok(run_adb_command_async(&["disconnect"], &state).await)
    }
}

#[tauri::command]
async fn get_devices(state: State<'_, AppState>) -> Result<Vec<Device>, ()> {
    let result = run_adb_command_async(&["devices", "-l"], &state).await;
    let mut devices = Vec::new();

    if !result.success {
        return Ok(devices);
    }

    for line in result.output.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let device_id = parts[0].to_string();
            let status = parts[1].to_string();
            let mut model = String::new();

            for part in parts.iter().skip(2) {
                if part.starts_with("model:") {
                    model = part[6..].to_string();
                    break;
                }
            }

            if model.is_empty() {
                let model_result = run_adb_command_async(&["-s", &device_id, "shell", "getprop", "ro.product.model"], &state).await;
                model = if model_result.success {
                    model_result.output.trim().to_string()
                } else {
                    "Unknown".to_string()
                };
            }

            devices.push(Device {
                device_id,
                status,
                model,
            });
        }
    }

    Ok(devices)
}

#[tauri::command]
async fn install_apk(device_id: String, apk_path: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    if !Path::new(&apk_path).exists() {
        return Ok(CommandResult {
            success: false,
            output: format!("APK file not found: {}", apk_path),
        });
    }
    Ok(run_adb_command_async(&["-s", &device_id, "install", "-r", &apk_path], &state).await)
}

#[tauri::command]
async fn hide_status_bar(device_id: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    Ok(run_adb_command_async(&["-s", &device_id, "shell", "wm", "overscan", "0,-50,0,0"], &state).await)
}

#[tauri::command]
async fn show_status_bar(device_id: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    Ok(run_adb_command_async(&["-s", &device_id, "shell", "wm", "overscan", "0,0,0,0"], &state).await)
}

#[tauri::command]
async fn hide_navigation_bar(device_id: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    Ok(run_adb_command_async(&["-s", &device_id, "shell", "wm", "overscan", "0,0,0,-100"], &state).await)
}

#[tauri::command]
async fn show_navigation_bar(device_id: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    Ok(run_adb_command_async(&["-s", &device_id, "shell", "wm", "overscan", "0,0,0,0"], &state).await)
}

#[tauri::command]
async fn hide_both_bars(device_id: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    Ok(run_adb_command_async(&["-s", &device_id, "shell", "wm", "overscan", "0,-50,0,-100"], &state).await)
}

#[tauri::command]
async fn show_both_bars(device_id: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    Ok(run_adb_command_async(&["-s", &device_id, "shell", "wm", "overscan", "0,0,0,0"], &state).await)
}

#[tauri::command]
async fn get_bar_status(device_id: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    Ok(run_adb_command_async(&["-s", &device_id, "shell", "wm", "overscan"], &state).await)
}

#[tauri::command]
async fn upload_file(device_id: String, local_path: String, remote_path: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    if !Path::new(&local_path).exists() {
        return Ok(CommandResult {
            success: false,
            output: format!("Local file not found: {}", local_path),
        });
    }
    Ok(run_adb_command_async(&["-s", &device_id, "push", &local_path, &remote_path], &state).await)
}

#[tauri::command]
async fn download_file(device_id: String, remote_path: String, local_path: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    Ok(run_adb_command_async(&["-s", &device_id, "pull", &remote_path, &local_path], &state).await)
}

#[tauri::command]
async fn list_android_directory(device_id: String, path: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    Ok(run_adb_command_async(&["-s", &device_id, "shell", "ls", "-la", &path], &state).await)
}

#[tauri::command]
async fn batch_delete(device_id: String, path: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    // First check if path exists
    let check_result = run_adb_command_async(&["-s", &device_id, "shell", "test", "-e", &path, "&&", "echo", "EXISTS", "||", "echo", "NOT_EXISTS"], &state).await;
    
    if check_result.success {
        let output = check_result.output.trim();
        if output == "NOT_EXISTS" {
            return Ok(CommandResult {
                success: false,
                output: "文件或文件夹不存在".to_string(),
            });
        }
    }
    
    // Check if it's a directory
    let dir_check = run_adb_command_async(&["-s", &device_id, "shell", "test", "-d", &path, "&&", "echo", "IS_DIR", "||", "echo", "IS_FILE"], &state).await;
    
    let is_directory = dir_check.success && dir_check.output.trim() == "IS_DIR";
    
    // Use appropriate delete command
    let delete_args = if is_directory {
        vec!["-s", &device_id, "shell", "rm", "-rf", &path]
    } else {
        vec!["-s", &device_id, "shell", "rm", "-f", &path]
    };
    
    let result = run_adb_command_async(&delete_args, &state).await;
    
    // Verify deletion
    let verify_result = run_adb_command_async(&["-s", &device_id, "shell", "test", "-e", &path, "&&", "echo", "EXISTS", "||", "echo", "DELETED"], &state).await;
    
    if verify_result.success && verify_result.output.trim() == "DELETED" {
        Ok(CommandResult {
            success: true,
            output: if is_directory { "文件夹删除成功".to_string() } else { "文件删除成功".to_string() },
        })
    } else {
        Ok(CommandResult {
            success: false,
            output: if result.output.is_empty() { "删除失败".to_string() } else { result.output },
        })
    }
}

#[tauri::command]
async fn execute_adb_command(device_id: String, command: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Ok(CommandResult {
            success: false,
            output: "命令不能为空".to_string(),
        });
    }

    let mut args = vec!["-s", &device_id];
    args.extend(parts);

    let result = run_adb_command_async(&args, &state).await;

    Ok(result)
}

#[tauri::command]
async fn open_adb_shell(device_id: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    let adb_path = {
        let path = state.adb_path.lock().unwrap();
        path.clone()
    };

    if adb_path.is_none() || adb_path.as_ref().unwrap().is_empty() {
        return Ok(CommandResult {
            success: false,
            output: "ADB路径未设置".to_string(),
        });
    }

    let adb_path_str = adb_path.unwrap();

    std::process::Command::new("cmd")
        .args(["/c", "start", "cmd", "/k", &adb_path_str, "-s", &device_id, "shell"])
        .spawn()
        .map_err(|_| ())?;

    Ok(CommandResult {
        success: true,
        output: format!("已打开 ADB Shell 到 {}", device_id),
    })
}

fn find_scrcpy_internal(_exe_dir: &std::path::Path) -> Option<String> {
    match ensure_tools_extracted() {
        Ok(cache_dir) => {
            let scrcpy_exe = cache_dir.join("scrcpy.exe");
            if scrcpy_exe.exists() {
                Some(scrcpy_exe.to_string_lossy().to_string())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

#[tauri::command]
async fn reboot_device(device_id: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    Ok(run_adb_command_async(&["-s", &device_id, "shell", "reboot"], &state).await)
}

#[tauri::command]
async fn uninstall_apk(device_id: String, package_name: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    // Uninstall app using adb uninstall command
    // Using --user 0 to uninstall for the primary user
    let result = run_adb_command_async(&["-s", &device_id, "uninstall", "--user", "0", &package_name], &state).await;
    
    // If the first attempt fails, try without --user flag
    if !result.success {
        let result2 = run_adb_command_async(&["-s", &device_id, "uninstall", &package_name], &state).await;
        return Ok(result2);
    }
    
    Ok(result)
}

#[tauri::command]
async fn get_foreground_app(device_id: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    // Get current focus window and extract package name
    let result = run_adb_command_async(&["-s", &device_id, "shell", "dumpsys", "window", "|", "grep", "mCurrentFocus"], &state).await;
    
    if !result.success {
        return Ok(result);
    }
    
    // Parse output to extract package name
    let output = &result.output;
    if let Some(start) = output.find("mCurrentFocus") {
        let substr = &output[start..];
        if let Some(pkg_start) = substr.find('/') {
            let before_slash = &substr[..pkg_start];
            if let Some(space_pos) = before_slash.rfind(' ') {
                let package_name = &before_slash[space_pos+1..];
                return Ok(CommandResult {
                    success: true,
                    output: package_name.to_string(),
                });
            }
        }
    }
    
    Ok(CommandResult {
        success: false,
        output: "无法解析前台应用包名".to_string(),
    })
}

#[tauri::command]
async fn backup_app_config(device_id: String, package_name: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    // Backup multiple directories that may contain app data including WebView cache
    // Directories to backup:
    // 1. /data/data/{package} - main app data
    // 2. /data/app/{package} - base APK (if exists)
    // 3. /sdcard/Android/data/{package} - external app data (if exists)

    let backup_cmd = format!(
        r#"cd / && (su 0 tar -zcpf /sdcard/app_data_backup.tar.gz \
            /data/data/{package} \
            /data/app/{package} \
            /sdcard/Android/data/{package} \
            2>/dev/null || \
            tar -zcpf /sdcard/app_data_backup.tar.gz \
            /data/data/{package} \
            /data/app/{package} \
            /sdcard/Android/data/{package} \
            2>/dev/null) && echo "BACKUP_SUCCESS" || echo "BACKUP_FAILED""#,
        package = package_name
    );

    Ok(run_adb_command_async(&["-s", &device_id, "shell", &backup_cmd], &state).await)
}

#[tauri::command]
async fn pull_backup_file(device_id: String, local_path: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    let result = run_adb_command_async(&["-s", &device_id, "pull", "/sdcard/app_data_backup.tar.gz", &local_path], &state).await;
    let _ = run_adb_command_async(&["-s", &device_id, "shell", "rm", "-f", "/sdcard/app_data_backup.tar.gz"], &state).await;
    Ok(result)
}

#[tauri::command]
async fn restore_app_config(device_id: String, backup_file: String, package_name: String, state: State<'_, AppState>) -> Result<CommandResult, ()> {
    // Step 1: Push backup file to device
    let push_result = run_adb_command_async(&["-s", &device_id, "push", &backup_file, "/sdcard/app_data_restore.tar.gz"], &state).await;
    if !push_result.success {
        return Ok(push_result);
    }

    // Step 2: Extract backup with permissions preserved (-p flag)
    let extract_cmd = format!(
        r#"su 0 tar -zxvpf /sdcard/app_data_restore.tar.gz -C / 2>/dev/null || \
           tar -zxvpf /sdcard/app_data_restore.tar.gz -C / 2>/dev/null"#,
    );
    let _extract_result = run_adb_command_async(&["-s", &device_id, "shell", &extract_cmd], &state).await;

    // Step 3: Fix SELinux contexts (if available and running as root)
    let selinux_cmd = format!(
        r#"su 0 restorecon -RF /data/data/{package} 2>/dev/null; \
           su 0 restorecon -RF /data/app/{package} 2>/dev/null; \
           su 0 restorecon -RF /sdcard/Android/data/{package} 2>/dev/null; \
           true"#,
        package = package_name
    );
    let _ = run_adb_command_async(&["-s", &device_id, "shell", &selinux_cmd], &state).await;

    // Step 4: Fix ownership recursively for all restored directories
    let chown_cmd = format!(
        r#"su 0 chown -R $(ls -ld /data/data/{package} 2>/dev/null | awk '{{print $3":"$4}}') /data/data/{package} 2>/dev/null; \
           su 0 chmod -R 755 /data/data/{package} 2>/dev/null; \
           su 0 chown -R $(ls -ld /data/app/{package} 2>/dev/null | awk '{{print $3":"$4}}') /data/app/{package} 2>/dev/null; \
           su 0 chmod -R 644 /data/app/{package} 2>/dev/null; \
           su 0 chown -R $(ls -ld /sdcard/Android/data/{package} 2>/dev/null | awk '{{print $3":"$4}}') /sdcard/Android/data/{package} 2>/dev/null; \
           true"#,
        package = package_name
    );
    let _ = run_adb_command_async(&["-s", &device_id, "shell", &chown_cmd], &state).await;

    // Step 5: Also fix app_data_dir specifically for WebView and other caches
    let fix_cache_cmd = format!(
        r#"su 0 find /data/data/{package} -type d -exec chmod 755 {{}} \; 2>/dev/null; \
           su 0 find /data/data/{package} -type f -exec chmod 644 {{}} \; 2>/dev/null; \
           su 0 find /data/data/{package}/app_webview -type d -exec chmod 755 {{}} \; 2>/dev/null; \
           su 0 find /data/data/{package}/app_webview -type f -exec chmod 644 {{}} \; 2>/dev/null; \
           su 0 find /data/data/{package}/cache -type d -exec chmod 755 {{}} \; 2>/dev/null; \
           su 0 find /data/data/{package}/cache -type f -exec chmod 644 {{}} \; 2>/dev/null; \
           true"#,
        package = package_name
    );
    let _ = run_adb_command_async(&["-s", &device_id, "shell", &fix_cache_cmd], &state).await;

    // Step 6: Clean up temp file
    let _ = run_adb_command_async(&["-s", &device_id, "shell", "rm", "-f", "/sdcard/app_data_restore.tar.gz"], &state).await;

    Ok(CommandResult {
        success: true,
        output: format!("恢复完成（包含 WebView 缓存）"),    })
}

#[tauri::command]
async fn find_scrcpy() -> Result<Option<String>, ()> {
    let exe_dir = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    let exe_dir = match exe_dir.parent() {
        Some(p) => p.to_path_buf(),
        None => return Ok(None),
    };
    
    Ok(find_scrcpy_internal(&exe_dir))
}

#[tauri::command]
async fn start_screen_mirror(device_id: String, _state: State<'_, AppState>) -> Result<CommandResult, ()> {
    let exe_dir = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => {
            return Ok(CommandResult {
                success: false,
                output: "无法获取程序目录".to_string(),
            });
        }
    };
    let exe_dir = match exe_dir.parent() {
        Some(p) => p.to_path_buf(),
        None => {
            return Ok(CommandResult {
                success: false,
                output: "无法获取程序目录".to_string(),
            });
        }
    };
    
    let scrcpy_exe = match find_scrcpy_internal(&exe_dir) {
        Some(p) => p,
        None => {
            return Ok(CommandResult {
                success: false,
                output: "未找到 scrcpy.exe，请确保 scrcpy 文件夹在程序目录下".to_string(),
            });
        }
    };
    
    // Start scrcpy in a new process (hidden console window)
    let output = Command::new(&scrcpy_exe)
        .args(&["-s", &device_id, "--window-title", &format!("屏幕镜像 - {}", device_id)])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
    
    match output {
        Ok(_) => Ok(CommandResult {
            success: true,
            output: "屏幕镜像已启动".to_string(),
        }),
        Err(e) => Ok(CommandResult {
            success: false,
            output: format!("启动失败: {}", e),
        }),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            adb_path: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            find_adb,
            set_adb_path,
            get_adb_path,
            connect_device,
            disconnect_device,
            get_devices,
            install_apk,
            uninstall_apk,
            hide_status_bar,
            show_status_bar,
            hide_navigation_bar,
            show_navigation_bar,
            hide_both_bars,
            show_both_bars,
            get_bar_status,
            upload_file,
            download_file,
            list_android_directory,
            batch_delete,
            execute_adb_command,
            open_adb_shell,
            start_screen_mirror,
            find_scrcpy,
            reboot_device,
            get_foreground_app,
            backup_app_config,
            pull_backup_file,
            restore_app_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
