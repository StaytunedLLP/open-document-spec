fn http_get_string(url: &str) -> Result<String, String> {
    let resp = apply_auth(ureq::get(url), "application/vnd.github+json")
        .timeout(Duration::from_secs(30))
        .call()
        .map_err(|e| format!("HTTP GET {url}: {e}{}", auth_hint()))?;
    resp.into_string()
        .map_err(|e| format!("read response {url}: {e}"))
}
