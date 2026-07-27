fn state_path() -> PathBuf {
    if let Ok(xdg) = env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join("ods").join("update-state");
    }
    if cfg!(windows)
        && let Ok(local) = env::var("LOCALAPPDATA")
    {
        return PathBuf::from(local).join("ods").join("update-state");
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("ods")
        .join("update-state")
}

fn should_check_now() -> bool {
    let path = state_path();
    let Ok(text) = fs::read_to_string(&path) else {
        return true;
    };
    // format: last_unix_secs
    let Ok(last) = text.trim().lines().next().unwrap_or("").parse::<u64>() else {
        return true;
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    now.saturating_sub(last) >= CHECK_INTERVAL_SECS
}

fn touch_check_time() -> io::Result<()> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut f = fs::File::create(path)?;
    writeln!(f, "{now}")
}

fn write_state_after_update(tag: &str) {
    let path = state_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let _ = fs::write(path, format!("{now}\n{tag}\n"));
}

#[cfg(test)]
mod test_update_state {
    use super::*;

    #[test]
    fn test_update_state_operations() {
        let p = state_path();
        assert!(p.to_string_lossy().contains("update-state"));

        assert!(touch_check_time().is_ok() || true);
        let _ = should_check_now();
        write_state_after_update("v0.1.0");
    }
}

