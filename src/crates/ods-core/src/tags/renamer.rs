/// Report for tag rename across the workspace.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TagRenameReport {
    pub from: String,
    pub to: String,
    pub rewritten_files: Vec<PathBuf>,
    pub matched_docs: usize,
    pub dry_run: bool,
}
