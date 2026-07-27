/// Rewrite `from` → `to` in all document frontmatter tag lists.
/// Dry-run when `write` is false.
pub fn rename_tag_in_workspace(
    workspace: &Workspace,
    from: &str,
    to: &str,
    write: bool,
) -> io::Result<TagRenameReport> {
    let from_n = normalize_tag(from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty source tag"))?;
    let to_n = normalize_tag(to)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty target tag"))?;
    if from_n == to_n {
        return Ok(TagRenameReport {
            from: from_n,
            to: to_n,
            dry_run: !write,
            ..Default::default()
        });
    }

    let mut report = TagRenameReport {
        from: from_n.clone(),
        to: to_n.clone(),
        dry_run: !write,
        ..Default::default()
    };

    for document in &workspace.documents {
        let FrontmatterState::Parsed(fm) = &document.frontmatter else {
            continue;
        };
        if !fm.tags.iter().any(|t| t == &from_n) {
            continue;
        }
        report.matched_docs += 1;
        let text = fs::read_to_string(&document.path)?;
        let Some(new_text) = rewrite_tags_in_text(&text, &from_n, &to_n) else {
            continue;
        };
        if new_text != text {
            report.rewritten_files.push(document.path.clone());
            if write {
                fs::write(&document.path, new_text)?;
            }
        }
    }

    Ok(report)
}
