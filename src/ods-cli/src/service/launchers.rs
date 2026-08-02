// User-level OS service install for background `ods serve` (start/stop).

use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Stable short id for a workspace root path (unit/task name).
pub fn workspace_unit_id(root: &Path) -> String {
    let abs = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let s = abs.to_string_lossy();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn ods_binary() -> PathBuf {
    env::current_exe().unwrap_or_else(|_| PathBuf::from("ods"))
}

/// systemd user unit body.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn render_systemd_user_unit(root: &Path, ods_bin: &Path) -> String {
    let root = abs_display(root);
    let bin = abs_display(ods_bin);
    format!(
        r#"[Unit]
Description=Open Document Spec workspace watch ({root})
After=default.target

[Service]
Type=simple
ExecStart={bin} serve --root {root}
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
"#
    )
}

/// launchd LaunchAgent plist body.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn render_launchd_plist(label: &str, root: &Path, ods_bin: &Path, log_dir: &Path) -> String {
    let root = abs_display(root);
    let bin = abs_display(ods_bin);
    let log = abs_display(&log_dir.join("ods-serve.log"));
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{bin}</string>
    <string>serve</string>
    <string>--root</string>
    <string>{root}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>{log}</string>
  <key>StandardErrorPath</key>
  <string>{log}</string>
</dict>
</plist>
"#
    )
}

fn abs_display(path: &Path) -> String {
    let p = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    p.to_string_lossy().replace('\\', "/")
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn linux_unit_path(unit_id: &str) -> PathBuf {
    let base = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            env::var("HOME")
                .map(|h| PathBuf::from(h).join(".config"))
                .unwrap_or_else(|_| PathBuf::from(".config"))
        });
    base.join("systemd/user")
        .join(format!("ods-watch-{unit_id}.service"))
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn macos_plist_path(unit_id: &str) -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join("Library/LaunchAgents")
        .join(format!("llp.ods.watch.{unit_id}.plist"))
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn windows_task_name(unit_id: &str) -> String {
    format!("Open Document Spec Watch {unit_id}")
}

pub struct ServiceStatus {
    pub installed: bool,
    pub running: bool,
    pub detail: String,
}

pub fn start_service(root: &Path) -> io::Result<String> {
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let id = workspace_unit_id(&root);
    let bin = ods_binary();
    let bin = fs::canonicalize(&bin).unwrap_or(bin);

    #[cfg(target_os = "linux")]
    {
        let unit = linux_unit_path(&id);
        if let Some(parent) = unit.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&unit, render_systemd_user_unit(&root, &bin))?;
        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();
        let name = unit
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("ods-watch.service");
        let status = Command::new("systemctl")
            .args(["--user", "enable", "--now", name])
            .status();
        match status {
            Ok(s) if s.success() => Ok(format!(
                "started user service {} for {}",
                name,
                root.display()
            )),
            Ok(_) | Err(_) => Ok(format!(
                "wrote {} (systemctl enable failed or unavailable — start manually: systemctl --user start {})",
                unit.display(),
                name
            )),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let plist = macos_plist_path(&id);
        if let Some(parent) = plist.parent() {
            fs::create_dir_all(parent)?;
        }
        let log_dir =
            PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".into())).join(".ods/logs");
        fs::create_dir_all(&log_dir)?;
        let label = format!("llp.ods.watch.{id}");
        fs::write(&plist, render_launchd_plist(&label, &root, &bin, &log_dir))?;
        let _ = Command::new("launchctl")
            .args(["unload", &plist.display().to_string()])
            .status();
        let status = Command::new("launchctl")
            .args(["load", &plist.display().to_string()])
            .status();
        match status {
            Ok(s) if s.success() => {
                Ok(format!(
                    "started LaunchAgent {} for {}",
                    label,
                    root.display()
                ))
            }
            Ok(_) | Err(_) => {
                Ok(format!(
                    "wrote {} (launchctl load failed or unavailable — load manually)",
                    plist.display()
                ))
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let name = windows_task_name(&id);
        let bin_s = bin.display().to_string();
        let root_s = root.display().to_string();
        let tr = format!("\"{bin_s}\" serve --root \"{root_s}\"");
        let status = Command::new("schtasks")
            .args([
                "/Create", "/F", "/TN", &name, "/SC", "ONLOGON", "/RL", "LIMITED", "/TR", &tr,
            ])
            .status();
        let _ = Command::new("schtasks")
            .args(["/Run", "/TN", &name])
            .status();
        match status {
            Ok(s) if s.success() => {
                Ok(format!(
                    "started scheduled task {name} for {}",
                    root.display()
                ))
            }
            Ok(_) | Err(_) => {
                Ok(format!(
                    "scheduled task create may have failed for {name}; try elevated shell"
                ))
            }
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = (root, id, bin);
        Ok("service registration not supported on this OS".into())
    }
}

#[cfg(test)]
mod test_launchers {
    use super::*;

    #[test]
    fn test_launcher_helpers() {
        let root = Path::new("src/fixtures/ecommerce");
        let id = workspace_unit_id(root);
        assert!(!id.is_empty());

        let bin = ods_binary();
        let bin_s = bin.to_string_lossy();
        assert!(
            bin_s.contains("ods") || bin_s.contains("ods"),
            "expected ods or ods binary, got {bin_s}"
        );

        let linux_path = linux_unit_path(&id);
        assert!(linux_path.to_string_lossy().contains("ods-watch-"));

        let macos_path = macos_plist_path(&id);
        assert!(macos_path.to_string_lossy().contains("llp.ods.watch."));

        let win_task = windows_task_name(&id);
        assert!(win_task.contains("Open Document Spec Watch "));

        let status_res = start_service(root);
        assert!(status_res.is_ok());
    }
}

