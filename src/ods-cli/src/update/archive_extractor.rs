fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<(), String> {
    let file = fs::File::open(archive).map_err(|e| e.to_string())?;
    let dec = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(dec);
    tar.unpack(dest).map_err(|e| format!("extract tar.gz: {e}"))
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<(), String> {
    let file = fs::File::open(archive).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("open zip: {e}"))?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = file
            .enclosed_name()
            .ok_or_else(|| "unsafe zip path".to_string())?
            .to_path_buf();
        let out = dest.join(name);
        if file.is_dir() {
            fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut target = fs::File::create(&out).map_err(|e| e.to_string())?;
            io::copy(&mut file, &mut target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn find_cli_binary(root: &Path, windows: bool) -> Result<PathBuf, String> {
    let preferred = if windows {
        ["ods.exe", "ods.exe"]
    } else {
        ["ods", "ods"]
    };
    let mut found: Option<PathBuf> = None;
    let mut prefer: Option<PathBuf> = None;
    visit(root, &mut |p| {
        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            if name == preferred[0] {
                prefer = Some(p.to_path_buf());
            } else if name == preferred[1] {
                found = Some(p.to_path_buf());
            }
        }
    });
    prefer
        .or(found)
        .ok_or_else(|| format!("archive missing ods/ods under {}", root.display()))
}

/// Legacy name used by install_release.
fn find_ods_binary(root: &Path, windows: bool) -> Result<PathBuf, String> {
    find_cli_binary(root, windows)
}

fn visit(dir: &Path, f: &mut dyn FnMut(&Path)) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            visit(&p, f);
        } else {
            f(&p);
        }
    }
}

#[cfg(test)]
mod test_archive_extractor {
    use super::*;

    #[test]
    fn test_find_ods_binary_missing_and_found() {
        let td = tempfile::tempdir().unwrap();
        let path = td.path();
        assert!(find_ods_binary(path, false).is_err());
        assert!(find_ods_binary(path, true).is_err());

        let bin_path = path.join("ods");
        fs::write(&bin_path, "fake bin").unwrap();
        assert_eq!(find_ods_binary(path, false).unwrap(), bin_path);
        // legacy name still found if no ods
        let _ = fs::remove_file(&bin_path);
        let legacy = path.join("ods");
        fs::write(&legacy, "fake bin").unwrap();
        assert_eq!(find_ods_binary(path, false).unwrap(), legacy);

        let win_bin = path.join("sub").join("ods.exe");
        fs::create_dir_all(win_bin.parent().unwrap()).unwrap();
        fs::write(&win_bin, "fake exe").unwrap();
        assert_eq!(find_ods_binary(path, true).unwrap(), win_bin);
    }

    #[test]
    fn test_extract_invalid_paths() {
        let td = tempfile::tempdir().unwrap();
        let invalid_file = td.path().join("nonexistent.tar.gz");
        assert!(extract_tar_gz(&invalid_file, td.path()).is_err());
        assert!(extract_zip(&invalid_file, td.path()).is_err());
    }
}

