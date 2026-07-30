/// Current process RSS in KB, using a native OS API where available so this
/// works without shelling out (and without depending on `ps` existing).
#[cfg(target_os = "linux")]
fn current_rss_kb() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        let rest = line.strip_prefix("VmRSS:")?;
        rest.split_whitespace().next()?.parse().ok()
    })
}

#[cfg(target_os = "windows")]
fn current_rss_kb() -> Option<u64> {
    let pid = std::process::id().to_string();
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    if text.contains("INFO:") || text.is_empty() {
        return None;
    }
    let last_col = text.rsplit(',').next()?;
    let digits: String = last_col.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

#[cfg(target_os = "macos")]
fn current_rss_kb() -> Option<u64> {
    // No cheap native syscall without extra bindings; shell out to `ps`,
    // which ships with every macOS install.
    let pid = std::process::id().to_string();
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn current_rss_kb() -> Option<u64> {
    None
}
