

/// Strip frontmatter across Markdown documents in workspace while generating a JSON snapshot backup.
pub fn bench_strip_workspace(
    root: &Path,
    options: BenchStripOptions,
) -> io::Result<BenchStripReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let workspace = load_workspace(&root)?;
    let backup_dir = get_backup_dir(&root)?;

    let now_sec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let snapshot_id = format!("snapshot-{now_sec}");
    let snapshot_file = backup_dir.join(format!("{snapshot_id}.json"));

    let mut file_entries = Vec::new();
    let mut deleted_indexes = Vec::new();
    let mut profile_files = Vec::new();
    let mut total_stripped = 0;
    let mut total_processed = 0;

    for doc in &workspace.documents {
        if let Some(ref filter) = options.path_filter {
            if !doc.path.starts_with(filter) {
                continue;
            }
        }
        total_processed += 1;

        let raw_text = fs::read_to_string(&doc.path)?;
        let (fm_str, body) = split_frontmatter(&raw_text);

        let rel_path = doc
            .path
            .strip_prefix(&root)
            .unwrap_or(&doc.path)
            .to_string_lossy()
            .to_string();

        file_entries.push(FileEntry {
            path: rel_path,
            frontmatter: fm_str.map(String::from),
        });

        if fm_str.is_some() {
            total_stripped += 1;
            if options.write {
                fs::write(&doc.path, body.trim_start_matches('\n'))?;
            }
        }
    }

    let mut total_indexes_deleted = 0;
    if options.strip_indexes || options.full {
        for path in collect_files_recursive(&root) {
            if path.file_name().is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("index.md") || n.to_string_lossy().eq_ignore_ascii_case("index.ods.md")) {
                let rel = path
                    .strip_prefix(&root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                if let Ok(content) = fs::read_to_string(&path) {
                    deleted_indexes.push(ContentEntry { path: rel, content });
                    total_indexes_deleted += 1;
                    if options.write {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }

    let mut total_profiles_removed = 0;
    if options.strip_profiles || options.full {
        let prof_roots = crate::profiles::profile_catalog_roots_from_config(&root, &workspace.config);
        for dir_name in &prof_roots {
            if dir_name.exists() {
                for path in collect_files_recursive(dir_name) {
                    let rel = path
                        .strip_prefix(&root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    if let Ok(content) = fs::read_to_string(&path) {
                        profile_files.push(ContentEntry { path: rel, content });
                        total_profiles_removed += 1;
                        if options.write {
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }

    let error_file = if options.full {
        let err_path = root.join(".ods").join("ods-errors.md");
        if err_path.exists() {
            let content = fs::read_to_string(&err_path).ok();
            if options.write {
                let _ = fs::remove_file(&err_path);
            }
            content
        } else {
            None
        }
    } else {
        None
    };

    let snapshot_data = Snapshot {
        snapshot_id: snapshot_id.clone(),
        root: root.to_string_lossy().to_string(),
        files: file_entries,
        deleted_indexes,
        profile_files,
        error_file,
    };

    if options.write {
        let json = serde_json::to_string_pretty(&snapshot_data)?;
        fs::write(&snapshot_file, json)?;
    }

    Ok(BenchStripReport {
        snapshot_id,
        snapshot_path: snapshot_file,
        total_processed,
        total_stripped,
        total_indexes_deleted,
        total_profiles_removed,
        dry_run: !options.write,
    })
}

/// Restore workspace frontmatters, index lockfiles, and profiles from a benchmark snapshot.
pub fn bench_restore_workspace(
    root: &Path,
    snapshot_id: Option<&str>,
) -> io::Result<BenchRestoreReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let backup_dir = get_backup_dir(&root)?;

    let target_snapshot_path = match snapshot_id {
        Some(id) => backup_dir.join(if id.ends_with(".json") {
            id.to_string()
        } else {
            format!("{id}.json")
        }),
        None => {
            let mut entries: Vec<PathBuf> = fs::read_dir(&backup_dir)?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
                .collect();
            entries.sort();
            entries
                .pop()
                .ok_or_else(|| io::Error::other("No ODS benchmark snapshots found to restore"))?
        }
    };

    let json_text = fs::read_to_string(&target_snapshot_path)?;
    let parsed_data: Snapshot = serde_json::from_str(&json_text)
        .map_err(|e| io::Error::other(format!("Corrupt snapshot JSON file: {e}")))?;

    let snapshot_id_final = parsed_data.snapshot_id;
    let mut total_restored = 0;

    for entry in parsed_data.files {
        let full_path = root.join(&entry.path);
        if !full_path.exists() {
            continue;
        }

        if let Some(fm) = entry.frontmatter {
            let current_body = fs::read_to_string(&full_path)?;
            let (_, body_only) = split_frontmatter(&current_body);

            let restored_text = format!("---\n{fm}---\n\n{}", body_only.trim_start_matches('\n'));
            fs::write(&full_path, restored_text)?;
            total_restored += 1;
        }
    }

    let mut total_indexes_restored = 0;
    for entry in parsed_data.deleted_indexes {
        let full_path = root.join(&entry.path);
        if let Some(parent) = full_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&full_path, entry.content)?;
        total_indexes_restored += 1;
    }

    let mut total_profiles_restored = 0;
    for entry in parsed_data.profile_files {
        let full_path = root.join(&entry.path);
        if let Some(parent) = full_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&full_path, entry.content)?;
        total_profiles_restored += 1;
    }

    if let Some(err_content) = parsed_data.error_file {
        let err_path = root.join(".ods").join("ods-errors.md");
        if let Some(parent) = err_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(err_path, err_content);
    }

    Ok(BenchRestoreReport {
        snapshot_id: snapshot_id_final,
        snapshot_path: target_snapshot_path,
        total_restored,
        total_indexes_restored,
        total_profiles_restored,
    })
}

fn collect_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_file() {
        files.push(dir.to_path_buf());
        return files;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_files_recursive(&path));
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    files
}

/// Create a snapshot backup of current workspace frontmatter and files before write operations.
pub fn create_workspace_snapshot(root: &Path) -> io::Result<BenchStripReport> {
    bench_strip_workspace(
        root,
        BenchStripOptions {
            write: false,
            path_filter: None,
            strip_indexes: false,
            strip_profiles: false,
            full: false,
        },
    )
}

/// Undo the latest write operation by restoring the most recent snapshot.
pub fn undo_latest_snapshot(root: &Path) -> io::Result<BenchRestoreReport> {
    bench_restore_workspace(root, None)
}

/// List snapshot ids (newest last) under the machine backup dir for this workspace.
pub fn list_workspace_snapshots(root: &Path) -> io::Result<Vec<String>> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let backup_dir = get_backup_dir(&root)?;
    if !backup_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<String> = fs::read_dir(&backup_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .filter_map(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect();
    entries.sort();
    Ok(entries)
}


