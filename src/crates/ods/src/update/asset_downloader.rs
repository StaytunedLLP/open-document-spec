/// Download a release asset by numeric id (private-repo safe).
fn http_get_asset(asset_id: u64) -> Result<Vec<u8>, String> {
    let url = format!("{API_BASE}/releases/assets/{asset_id}");
    let resp = apply_auth(ureq::get(&url), "application/octet-stream")
        .timeout(Duration::from_secs(300))
        .call()
        .map_err(|e| format!("HTTP GET {url}: {e}{}", auth_hint()))?;
    let mut out = Vec::new();
    resp.into_reader()
        .read_to_end(&mut out)
        .map_err(|e| format!("download asset {asset_id}: {e}"))?;
    Ok(out)
}
