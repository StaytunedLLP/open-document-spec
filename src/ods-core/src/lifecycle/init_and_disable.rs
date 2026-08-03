// Opt-in init / opt-out disable for ODS workspaces.
//
// Init is explicit (`ods init`). Disable strips ODS metadata and leaves prose intact.

use crate::adopt::{AdoptOptions, adopt_workspace};
use crate::fs::{find_workspace_root, index_has_ods_field, load_workspace};
use crate::index::generate_indexes;
use crate::model::current_ods_spec_version;
use crate::parse::split_frontmatter;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// True when `root` has `index.ods.md` declaring `ods:`.
pub fn ods_enabled(root: impl AsRef<Path>) -> bool {
    let root = root.as_ref();
    let index_ods = root.join("index.ods.md");
    if index_ods.is_file() && index_has_ods_field(&index_ods) {
        return true;
    }
    let index_md = root.join("index.md");
    index_md.is_file() && index_has_ods_field(&index_md)
}

/// Resolve whether ODS is enabled for a path (file or directory).
pub fn ods_enabled_for_path(path: impl AsRef<Path>) -> bool {
    find_workspace_root(path.as_ref())
        .map(ods_enabled)
        .unwrap_or(false)
}

const DOC_ODS_KEYS: &[&str] = &[
    "ods",
    "profile",
    "status",
    "share",
    "id",
    "description",
    "depends",
    "related",
    "resources",
    "code",
    "context",
    "owner",
    "tags",
];

const ROOT_ODS_KEYS: &[&str] = &["ods", "ods", "profiles", "custom-profiles", "ignore", "aliases"];

/// Options for `disable_workspace`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisableOptions {
    /// Apply changes (default false = dry-run).
    pub write: bool,
    /// Strip ODS frontmatter keys from documents (default true).
    pub strip_frontmatter: bool,
    /// Strip root policy keys ods/profiles/ignore/aliases (default true).
    pub strip_root_policy: bool,
    /// Delete non-root index.md files (default false; otherwise only strip FM).
    pub remove_indexes: bool,
    /// Delete root index.md (default false; dangerous).
    pub remove_root_index: bool,
}

impl Default for DisableOptions {
    fn default() -> Self {
        Self {
            write: false,
            strip_frontmatter: true,
            strip_root_policy: true,
            remove_indexes: false,
            remove_root_index: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DisableReport {
    pub root: PathBuf,
    pub already_disabled: bool,
    pub would_edit: Vec<PathBuf>,
    pub edited: Vec<PathBuf>,
    pub would_delete: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
    pub dry_run: bool,
}

/// Options for [`init_workspace`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InitOptions {
    /// Run adopt --write after ensuring root marker.
    pub adopt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InitReport {
    pub root: PathBuf,
    /// Created root index or injected `ods:`.
    pub initialized: bool,
    /// Root already had `ods:`.
    pub already_initialized: bool,
    pub adopted: Vec<PathBuf>,
    pub indexes: Vec<PathBuf>,
}

/// Ensure workspace has `ods:` root, optionally adopt plain files, generate indexes.
///
/// Single opt-in path for `ods init` (replaces the former `enable` command).
pub fn init_workspace(root: impl AsRef<Path>, options: InitOptions) -> io::Result<InitReport> {
    let root = canonical_or_original(root.as_ref());
    fs::create_dir_all(&root)?;
    let mut report = InitReport {
        root: root.clone(),
        ..Default::default()
    };

    let index = root.join("index.ods.md");
    if ods_enabled(&root) {
        let text = fs::read_to_string(&index)?;
        let next = ensure_ods_in_index_text(&text);
        if next != text {
            fs::write(&index, next)?;
            report.initialized = true;
        } else {
            report.already_initialized = true;
        }
    } else if index.exists() {
        // Inject or update ods: in existing root index frontmatter or prepend block.
        let text = fs::read_to_string(&index)?;
        let next = ensure_ods_in_index_text(&text);
        if next != text {
            fs::write(&index, next)?;
            report.initialized = true;
        } else {
            report.already_initialized = true;
        }
    } else {
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Workspace");
        let content = format!(
            "---\nprofile: index\nods: {}\n---\n\n# {name}\n\n",
            current_ods_spec_version()
        );
        fs::write(&index, content)?;
        report.initialized = true;
    }

    let workspace = load_workspace(&root)?;
    // `adopt_workspace(write: true)` rewrites files, so only reload when it ran;
    // otherwise reuse the workspace already loaded above (avoids a redundant
    // same-state re-parse of the whole tree).
    let workspace = if options.adopt {
        let adopt_report = adopt_workspace(&workspace, AdoptOptions { write: true })?;
        report.adopted = adopt_report.written;
        load_workspace(&root)?
    } else {
        workspace
    };
    report.indexes = generate_indexes(&workspace)?;
    Ok(report)
}

/// Dry-run or apply ODS disable / revert to plain Markdown metadata.
pub fn disable_workspace(
    root: impl AsRef<Path>,
    options: DisableOptions,
) -> io::Result<DisableReport> {
    let root = canonical_or_original(root.as_ref());
    let mut report = DisableReport {
        root: root.clone(),
        dry_run: !options.write,
        ..Default::default()
    };

    if !ods_enabled(&root) {
        // Still allow stripping if user pointed at a tree with frontmatter but no ods:
        // Prefer strict: already disabled when no ods: marker.
        report.already_disabled = true;
        return Ok(report);
    }

    let workspace = load_workspace(&root)?;
    let root_index = root.join("index.ods.md");

    for document in &workspace.documents {
        let path = &document.path;
        let is_root_index = path == &root_index;
        let is_index = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == "index.md" || n == "index.ods.md");

        if options.remove_indexes && is_index && !is_root_index {
            report.would_delete.push(path.clone());
            if options.write {
                fs::remove_file(path)?;
                report.deleted.push(path.clone());
            }
            continue;
        }

        if options.remove_root_index && is_root_index {
            report.would_delete.push(path.clone());
            if options.write {
                fs::remove_file(path)?;
                report.deleted.push(path.clone());
            }
            continue;
        }

        if !(options.strip_frontmatter || (is_root_index && options.strip_root_policy)) {
            continue;
        }

        let text = fs::read_to_string(path)?;
        let strip_doc = options.strip_frontmatter;
        let strip_root = is_root_index && options.strip_root_policy;
        let (next, changed) = strip_ods_from_document_text(&text, strip_doc, strip_root);
        if !changed {
            continue;
        }
        // Body must be unchanged
        let (_, body_before) = split_frontmatter(&text);
        let (_, body_after) = split_frontmatter(&next);
        let body_before = body_before.trim_start_matches(['\r', '\n']);
        let body_after = body_after.trim_start_matches(['\r', '\n']);
        if body_before != body_after {
            return Err(io::Error::other(format!(
                "refuse to change body of {}",
                path.display()
            )));
        }

        report.would_edit.push(path.clone());
        if options.write {
            fs::write(path, next)?;
            report.edited.push(path.clone());
        }
    }

    Ok(report)
}

fn canonical_or_original(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
