fn http_get_string(url: &str) -> Result<String, String> {
    let resp = apply_auth(ureq::get(url), "application/vnd.github+json")
        .timeout(Duration::from_secs(30))
        .call()
        .map_err(|e| format!("HTTP GET {url}: {e}"))?;
    resp.into_string()
        .map_err(|e| format!("read response {url}: {e}"))
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>, String> {
    let resp = apply_auth(ureq::get(url), "application/octet-stream")
        .timeout(Duration::from_secs(300))
        .call()
        .map_err(|e| format!("HTTP GET {url}: {e}"))?;
    let mut out = Vec::new();
    resp.into_reader()
        .read_to_end(&mut out)
        .map_err(|e| format!("read response {url}: {e}"))?;
    Ok(out)
}

