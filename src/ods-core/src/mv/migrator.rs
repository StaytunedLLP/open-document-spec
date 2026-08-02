/// Canonical order of ODS engine keys inside the nested `ods:` map
/// (specs/SPEC.md "Canonical Key Sequence Rule").
const CANONICAL_ODS_KEY_ORDER: [&str; 9] = [
    "profile",
    "status",
    "id",
    "share",
    "depends",
    "related",
    "resources",
    "code",
    "context",
];

/// Migrate one document's raw frontmatter text into canonical Pattern B
/// shape: universal top-level keys (`description`, `tags`, `owner`, and any
/// other non-engine top-level key) in their existing relative order,
/// followed by a single `ods:` block with engine keys
/// (`profile`/`status`/`id`/`share`/`depends`/`related`/`resources`/`code`/`context`)
/// in canonical order.
///
/// Operates on raw text/lines, never on the parsed [`crate::model::Frontmatter`]
/// struct, because that struct is lossy for `owner` and `code[].symbol`
/// (both collapse YAML list-vs-scalar form into a single joined string) —
/// re-emitting from the struct would silently corrupt those fields' original
/// shape. Idempotent: returns `None` if nothing changes.
///
/// Skips (returns `None` for) documents that use a scalar `ods: <version>`
/// marker line (the root `index.md` workspace-marker form) rather than a
/// nested `ods:` map, and documents with no frontmatter block at all.
pub fn migrate_frontmatter_to_canonical(text: &str) -> Option<String> {
    let (frontmatter, body) = crate::parse::split_frontmatter(text);
    let frontmatter = frontmatter?;

    if has_scalar_ods_marker(frontmatter) {
        return None;
    }

    let blocks = group_top_level_blocks(frontmatter);
    if blocks.is_empty() {
        return None;
    }

    let mut engine: std::collections::BTreeMap<&str, (usize, Vec<String>)> =
        std::collections::BTreeMap::new();

    for (position, block) in blocks.iter().enumerate() {
        if let Some(&canonical_key) = CANONICAL_ODS_KEY_ORDER.iter().find(|k| **k == block.key) {
            engine.insert(canonical_key, (position, reindent(&block.lines, 2)));
        } else if block.key == "ods" {
            for sub in group_sub_blocks(&block.lines[1..], 2) {
                if let Some(&canonical_key) =
                    CANONICAL_ODS_KEY_ORDER.iter().find(|k| **k == sub.key)
                {
                    let candidate_wins = match engine.get(canonical_key) {
                        Some((existing_position, _)) => position >= *existing_position,
                        None => true,
                    };
                    if candidate_wins {
                        engine.insert(canonical_key, (position, sub.lines));
                    }
                }
            }
        }
    }

    if engine.is_empty() {
        return None;
    }

    let mut new_frontmatter_lines: Vec<String> = Vec::new();
    for block in &blocks {
        let is_engine_key = CANONICAL_ODS_KEY_ORDER.contains(&block.key.as_str());
        if is_engine_key || block.key == "ods" {
            continue;
        }
        new_frontmatter_lines.extend(block.lines.iter().cloned());
    }

    new_frontmatter_lines.push("ods:".to_string());
    for key in CANONICAL_ODS_KEY_ORDER {
        if let Some((_, lines)) = engine.get(key) {
            new_frontmatter_lines.extend(lines.iter().cloned());
        }
    }

    let ending = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let new_frontmatter = new_frontmatter_lines.join(ending);
    let out = if body.is_empty() {
        format!("---{ending}{new_frontmatter}{ending}---{ending}")
    } else {
        format!("---{ending}{new_frontmatter}{ending}---{ending}{body}")
    };

    if out == text {
        None
    } else {
        Some(out)
    }
}

/// Rewrite every document under `root` into canonical Pattern B frontmatter
/// shape via [`migrate_frontmatter_to_canonical`]. Skips the workspace root
/// `index.md` (the scalar `ods: <version>` marker file) and any document
/// whose frontmatter failed to parse or is absent.
pub fn migrate_workspace_frontmatter(root: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let workspace = load_workspace(root.as_ref())?;
    migrate_workspace_frontmatter_with_workspace(&workspace)
}

/// Same as [`migrate_workspace_frontmatter`], but takes an already-loaded
/// `Workspace` instead of reloading — each document's text is still
/// re-read fresh from disk, so this is safe to run after
/// [`normalize_workspace_frontmatter_spacing_with_workspace`] and
/// [`canonicalize_workspace_document_refs_with_workspace`] against the same
/// workspace.
pub fn migrate_workspace_frontmatter_with_workspace(
    workspace: &crate::model::Workspace,
) -> io::Result<Vec<PathBuf>> {
    let root_index = workspace.root.join("index.md");
    let mut changed = Vec::new();

    for document in &workspace.documents {
        if document.path == root_index {
            continue;
        }
        if !matches!(
            document.frontmatter,
            crate::model::FrontmatterState::Parsed(_)
        ) {
            continue;
        }

        let text = match fs::read_to_string(&document.path) {
            Ok(text) => text,
            Err(_) => continue,
        };

        if let Some(next) = migrate_frontmatter_to_canonical(&text) {
            fs::write(&document.path, &next)?;
            changed.push(document.path.clone());
        }
    }

    Ok(changed)
}

struct Block {
    key: String,
    lines: Vec<String>,
}

/// True if `frontmatter` contains a top-level `ods: <value>` line (the root
/// workspace scalar version marker, e.g. `ods: 0.1`), as opposed to a bare
/// `ods:` line that introduces a nested map.
fn has_scalar_ods_marker(frontmatter: &str) -> bool {
    frontmatter.lines().any(|line| {
        indent(line) == 0
            && line
                .trim_start()
                .strip_prefix("ods:")
                .is_some_and(|rest| !rest.trim().is_empty())
    })
}

/// Group frontmatter lines into top-level (indent == 0) key blocks: a block
/// is its key line plus every following line up to (not including) the next
/// indent-0 line. Blank lines are dropped — they carry no semantic weight in
/// YAML frontmatter and `normalize_frontmatter_body_spacing` separately owns
/// edge-of-block spacing.
fn group_top_level_blocks(frontmatter: &str) -> Vec<Block> {
    group_blocks(frontmatter.lines(), 0)
}

/// Same grouping one level down: `min_indent`-indented key lines plus
/// deeper-indented continuation lines belong to the preceding key's block.
fn group_sub_blocks(lines: &[String], min_indent: usize) -> Vec<Block> {
    group_blocks(lines.iter().map(|s| s.as_str()), min_indent)
}

fn group_blocks<'a>(lines: impl Iterator<Item = &'a str>, key_indent: usize) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        if indent(line) == key_indent {
            let key = line
                .trim_start()
                .split_once(':')
                .map(|(k, _)| k.trim().to_string())
                .unwrap_or_else(|| line.trim().to_string());
            blocks.push(Block {
                key,
                lines: vec![line.to_string()],
            });
        } else if let Some(last) = blocks.last_mut() {
            last.lines.push(line.to_string());
        }
    }
    blocks
}

/// Prefix every line in `lines` with `extra` additional spaces of indentation.
fn reindent(lines: &[String], extra: usize) -> Vec<String> {
    let pad = " ".repeat(extra);
    lines.iter().map(|line| format!("{pad}{line}")).collect()
}

fn indent(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}
