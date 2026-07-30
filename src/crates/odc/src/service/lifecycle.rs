pub fn stop_service(root: &Path, unregister: bool) -> io::Result<String> {
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let id = workspace_unit_id(&root);

    #[cfg(target_os = "linux")]
    {
        let unit = linux_unit_path(&id);
        let name = unit
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("ods-watch.service");
        let _ = Command::new("systemctl")
            .args(["--user", "stop", name])
            .status();
        if unregister {
            let _ = Command::new("systemctl")
                .args(["--user", "disable", name])
                .status();
            let _ = fs::remove_file(&unit);
            return Ok(format!("stopped and unregistered {name}"));
        }
        Ok(format!("stopped {name}"))
    }

    #[cfg(target_os = "macos")]
    {
        let plist = macos_plist_path(&id);
        let _ = Command::new("launchctl")
            .args(["unload", &plist.display().to_string()])
            .status();
        if unregister {
            let _ = fs::remove_file(&plist);
            Ok(format!("stopped and removed {}", plist.display()))
        } else {
            Ok(format!("stopped {}", plist.display()))
        }
    }

    #[cfg(target_os = "windows")]
    {
        let name = windows_task_name(&id);
        let _ = Command::new("schtasks")
            .args(["/End", "/TN", &name])
            .status();
        if unregister {
            let _ = Command::new("schtasks")
                .args(["/Delete", "/F", "/TN", &name])
                .status();
            Ok(format!("stopped and deleted task {name}"))
        } else {
            Ok(format!("stopped task {name}"))
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = (root, id, unregister);
        Ok("service stop not supported on this OS".into())
    }
}

pub fn service_status(root: &Path) -> ServiceStatus {
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let id = workspace_unit_id(&root);

    #[cfg(target_os = "linux")]
    {
        let unit = linux_unit_path(&id);
        let installed = unit.is_file();
        let name = unit
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("ods-watch.service");
        let running = Command::new("systemctl")
            .args(["--user", "is-active", name])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
            .unwrap_or(false);
        ServiceStatus {
            installed,
            running,
            detail: format!("unit {}", unit.display()),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let plist = macos_plist_path(&id);
        let label = format!("llp.odc.watch.{id}");
        let installed = plist.is_file();
        let running = Command::new("launchctl")
            .args(["list", &label])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        ServiceStatus {
            installed,
            running,
            detail: format!("plist {}", plist.display()),
        }
    }

    #[cfg(target_os = "windows")]
    {
        let name = windows_task_name(&id);
        let installed = Command::new("schtasks")
            .args(["/Query", "/TN", &name])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        ServiceStatus {
            installed,
            running: installed,
            detail: format!("task {name}"),
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = id;
        ServiceStatus {
            installed: false,
            running: false,
            detail: "unsupported OS".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemd_unit_contains_serve_and_root() {
        let root = Path::new("/tmp/odc-ws");
        let bin = Path::new("/usr/local/bin/odc");
        let u = render_systemd_user_unit(root, bin);
        assert!(u.contains("serve"));
        assert!(u.contains("--root"));
        assert!(u.contains("odc"));
    }

    #[test]
    fn launchd_plist_contains_label_and_args() {
        let root = Path::new("/Users/me/docs");
        let bin = Path::new("/Users/me/.local/bin/odc");
        let log = Path::new("/Users/me/.odc/logs");
        let p = render_launchd_plist("llp.odc.watch.abc", root, bin, log);
        assert!(p.contains("llp.odc.watch.abc"));
        assert!(p.contains("serve"));
        assert!(p.contains("--root"));
    }
}
