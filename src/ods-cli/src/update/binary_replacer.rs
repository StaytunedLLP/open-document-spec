fn replace_binary(src: &Path, dest: &Path) -> Result<(), String> {
    let bytes =
        fs::read(src).map_err(|e| ods_core::error::detail(&format!("read {}", src.display()), e))?;
    let parent = dest
        .parent()
        .ok_or_else(|| "invalid install path".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|e| ods_core::error::detail("create install parent", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let tmp = dest.with_extension("ods-new");
        fs::write(&tmp, &bytes)
            .map_err(|e| ods_core::error::detail(&format!("write {}", tmp.display()), e))?;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755))
            .map_err(|e| ods_core::error::detail(&format!("chmod {}", tmp.display()), e))?;
        fs::rename(&tmp, dest).map_err(|e| {
            format!(
                "install {}: {e} (is the directory writable?)",
                dest.display()
            )
        })?;
    }

    #[cfg(windows)]
    {
        let tmp = dest.with_extension("ods-new");
        let old = dest.with_extension("ods-old");
        fs::write(&tmp, &bytes)
            .map_err(|e| ods_core::error::detail(&format!("write {}", tmp.display()), e))?;
        let _ = fs::remove_file(&old);
        if dest.exists() {
            fs::rename(dest, &old).map_err(|e| {
                format!(
                    "replace {}: {e} (close running ods processes and retry)",
                    dest.display()
                )
            })?;
        }
        fs::rename(&tmp, dest)
            .map_err(|e| ods_core::error::detail(&format!("install {}", dest.display()), e))?;
        let _ = fs::remove_file(&old);
    }

    #[cfg(not(any(unix, windows)))]
    {
        fs::write(dest, &bytes)
            .map_err(|e| ods_core::error::detail("write binary", e))?;
    }

    Ok(())
}

fn install_prefix() -> Result<PathBuf, String> {
    let exe =
        env::current_exe().map_err(|e| ods_core::error::detail("current_exe", e))?;
    let exe = fs::canonicalize(&exe).unwrap_or(exe);
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "cannot determine install directory".into())
}

#[cfg(test)]
mod test_binary_replacer {
    use super::*;

    #[test]
    fn test_replace_binary_and_install_prefix() {
        let td = tempfile::tempdir().unwrap();
        let src = td.path().join("source_bin");
        let dest = td.path().join("dest_dir").join("ods_target");

        fs::write(&src, b"binary_content").unwrap();
        assert!(replace_binary(&src, &dest).is_ok());
        assert_eq!(fs::read(&dest).unwrap(), b"binary_content");

        assert!(install_prefix().is_ok());
    }
}

