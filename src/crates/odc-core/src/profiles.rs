use crate::model::{Document, FrontmatterState, ProfileCatalog, ProfileDefinition};
use crate::parse::{extract_heading_groups, parse_document_text};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn standard_profile_catalog() -> ProfileCatalog {
    let mut catalog = ProfileCatalog::default();
    for definition in standard_profile_definitions() {
        catalog
            .definitions
            .insert(definition.name.clone(), definition);
    }
    catalog
}

pub fn profile_catalog_roots(root: &Path, root_index: Option<&Document>) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(root_index) = root_index
        && let FrontmatterState::Parsed(frontmatter) = &root_index.frontmatter
    {
        roots.extend(frontmatter.profiles.iter().map(|path| root.join(path)));

        // Also check imported ODS Packs listed under packs:
        for pack_ref in &frontmatter.packs {
            let pack_dir = root.join(pack_ref);
            let pack_dir = pack_dir.canonicalize().unwrap_or(pack_dir);
            let pack_profiles = pack_dir.join("ods-profiles");
            if pack_profiles.exists() {
                roots.push(pack_profiles);
            } else if pack_dir.exists() {
                roots.push(pack_dir);
            }
        }
    }

    let default_root = root.join("ods-profiles");
    if default_root.exists() {
        roots.push(default_root);
    }

    roots.sort();
    roots.dedup();
    roots
}

pub fn load_profile_catalog(root: &Path, roots: &[PathBuf]) -> io::Result<ProfileCatalog> {
    let mut catalog = standard_profile_catalog();

    for profile_root in roots {
        if !profile_root.exists() {
            continue;
        }

        let mut paths = Vec::new();
        collect_markdown_paths(profile_root, &mut paths)?;
        paths.sort();

        for path in paths {
            let text = fs::read_to_string(&path)?;
            let document = parse_document_text(root, path.clone(), &text, true);
            if let Some(definition) = profile_definition_from_document(&document) {
                if let Some(existing) = catalog.definitions.get(&definition.name) {
                    catalog.conflicts.push(crate::model::ProfileConflict {
                        name: definition.name.clone(),
                        kept: existing.source.clone(),
                        ignored: definition.source.clone(),
                    });
                } else {
                    catalog
                        .definitions
                        .insert(definition.name.clone(), definition);
                }
            }
        }
    }

    Ok(catalog)
}

fn collect_markdown_paths(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_type = entry.file_type()?;

        if should_ignore_name(&file_name) {
            continue;
        }

        if file_type.is_dir() {
            collect_markdown_paths(&path, out)?;
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }

    Ok(())
}

fn should_ignore_name(name: &std::ffi::OsStr) -> bool {
    let text = name.to_string_lossy();
    text.starts_with('.') || text == "target"
}

fn profile_definition_from_document(document: &Document) -> Option<ProfileDefinition> {
    let name = profile_name_from_path(&document.path)?;
    let mut sections = extract_heading_groups(&document.body);

    if let FrontmatterState::Parsed(frontmatter) = &document.frontmatter {
        for (canonical, aliases) in &frontmatter.aliases {
            if let Some(group) = sections
                .iter_mut()
                .find(|group| group.first() == Some(canonical))
            {
                for alias in aliases {
                    if !group.contains(alias) {
                        group.push(alias.clone());
                    }
                }
            } else {
                let mut group = vec![canonical.clone()];
                for alias in aliases {
                    if !group.contains(alias) {
                        group.push(alias.clone());
                    }
                }
                sections.push(group);
            }
        }
    }

    Some(ProfileDefinition {
        name,
        sections,
        source: document.path.clone(),
    })
}

fn profile_name_from_path(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?.to_string();
    if stem == "index" {
        path.parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .or(Some(stem))
    } else {
        Some(stem)
    }
}

fn standard_profile_definitions() -> Vec<ProfileDefinition> {
    vec![
        profile("note", vec![]),
        profile(
            "feature",
            vec![
                section(&["Goal", "Objective", "Objectives", "Purpose"]),
                section(&["Scope", "In Scope", "Boundaries"]),
                section(&["Requirements", "Functional Requirements", "Needs"]),
                section(&[
                    "Acceptance Criteria",
                    "Acceptance",
                    "Success Criteria",
                    "Definition of Done",
                ]),
                section(&["Risks", "Risks and Mitigations", "Concerns"]),
            ],
        ),
        profile(
            "guide",
            vec![
                section(&["Overview", "Introduction", "Summary", "Background"]),
                section(&["Prerequisites", "Requirements", "Before You Begin"]),
                section(&["Steps", "Instructions", "Procedure", "Process"]),
                section(&["Troubleshooting", "Common Issues", "FAQ"]),
            ],
        ),
        profile(
            "api",
            vec![
                section(&["Overview", "Introduction", "Summary", "Background"]),
                section(&["Request"]),
                section(&["Response"]),
                section(&["Errors"]),
                section(&["Examples"]),
            ],
        ),
        profile(
            "architecture",
            vec![
                section(&["Overview", "Introduction", "Summary", "Background"]),
                section(&["Components"]),
                section(&["Data Flow"]),
                section(&["Trade-offs", "Tradeoffs", "Pros and Cons"]),
            ],
        ),
        profile(
            "decision",
            vec![
                section(&["Context", "Background"]),
                section(&["Decision"]),
                section(&["Alternatives", "Options", "Options Considered"]),
                section(&["Consequences", "Outcome", "Implications"]),
            ],
        ),
        profile(
            "sop",
            vec![
                section(&["Purpose"]),
                section(&["Prerequisites", "Requirements", "Before You Begin"]),
                section(&["Steps", "Instructions", "Procedure", "Process"]),
                section(&["Validation", "Verification", "Checks"]),
                section(&["Rollback", "Recovery", "Revert"]),
            ],
        ),
        profile(
            "policy",
            vec![
                section(&["Purpose"]),
                section(&["Scope"]),
                section(&["Rules", "Standards", "Requirements"]),
                section(&["Exceptions"]),
            ],
        ),
        profile(
            "meeting",
            vec![
                section(&["Attendees"]),
                section(&["Agenda"]),
                section(&["Decisions"]),
                section(&["Action Items", "Actions", "Next Steps", "TODO"]),
            ],
        ),
        profile("faq", vec![]),
        profile(
            "checklist",
            vec![
                section(&["Overview", "Purpose", "Introduction", "Summary"]),
                section(&["Items", "Checklist", "Tasks", "Steps"]),
                section(&[
                    "Verification",
                    "Done When",
                    "Acceptance",
                    "Definition of Done",
                    "Checks",
                ]),
                section(&["Notes", "Exceptions", "Caveats", "References"]),
            ],
        ),
        profile("index", vec![]),
    ]
}

fn profile(name: &str, sections: Vec<Vec<&str>>) -> ProfileDefinition {
    ProfileDefinition {
        name: name.to_string(),
        sections: sections
            .into_iter()
            .map(|group| group.into_iter().map(|value| value.to_string()).collect())
            .collect(),
        source: PathBuf::from(format!("<builtin:{name}>")),
    }
}

fn section<'a>(values: &'a [&'a str]) -> Vec<&'a str> {
    values.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_pack_and_alias_edge_cases() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        let pack_dir = root.join("pack_without_profiles");
        fs::create_dir_all(&pack_dir).unwrap();

        let index_doc = parse_document_text(root, root.join("index.md"), "---\npacks:\n  - pack_without_profiles\n---\n", true);
        let roots = profile_catalog_roots(root, Some(&index_doc));
        assert!(roots.contains(&pack_dir) || roots.contains(&root.join("ods-profiles")));

        let prof_dir = root.join("ods-profiles").join("sub");
        fs::create_dir_all(&prof_dir).unwrap();
        fs::write(prof_dir.join(".hidden"), "ignored").unwrap();
        fs::write(prof_dir.join("subprof.md"), "---\naliases:\n  NewCanonical:\n    - AliasOne\n---\n# Subprof\n").unwrap();

        let roots = profile_catalog_roots(root, None);
        let cat = load_profile_catalog(root, &roots).unwrap();
        assert!(cat.definitions.contains_key("subprof"));
    }
}
