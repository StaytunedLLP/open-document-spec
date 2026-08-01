---
description: "Diagnostic workflow, complete error catalog for .ods/ods-errors.md, git merge conflicts, and daemon troubleshooting."
status: "stable"
order: 7
ods:
  profile: "note"
  status: "stable"
---

# Troubleshooting and Diagnostics Guide

This guide provides a comprehensive reference for diagnosing workspace issues, resolving `ods lint` errors in `.ods/ods-errors.md`, handling Git merge conflicts, and troubleshooting background service daemons (`ods serve`).

---

## 1. The Diagnostic Workflow

When an issue occurs or documents are modified, follow this 3-step diagnostic workflow:

```bash
# Step 1: Check overall workspace health and daemon service status
ods doctor

# Step 2: Validate graph relationships, schemas, and references
ods lint --level 3

# Step 3: Check index lockfile freshness
ods index --check
```

If `ods lint` detects errors, it writes a detailed diagnostic report to `.ods/ods-errors.md`. When all issues are resolved, `ods lint` automatically deletes that report and outputs a green confirmation message.

---

## 2. Complete `.ods/ods-errors.md` Lint Error Catalog

Below is the complete catalog of errors and warnings reported by `ods lint`, along with their root causes and step-by-step resolution actions.

### 1. YAML Frontmatter Parse Error
* **Severity**: Error (Level 1+)
* **Diagnostic Message**: `Failed to parse YAML frontmatter: <syntax error details>`
* **Cause**: Invalid YAML syntax (e.g., unquoted string containing colons, bad indentation, tabs instead of spaces).
* **Resolution**: Fix the YAML formatting in the specified document. Use 2 spaces for indentation and wrap strings containing special characters in quotes.

### 2. Invalid Status Enum
* **Severity**: Error (Level 1+)
* **Diagnostic Message**: `invalid status value: '<value>'`
* **Cause**: `status` in frontmatter is set to an unsupported value (e.g. `done`, `wip`, `COMPLETE`).
* **Resolution**: Change `status` to one of the four valid lowercase enums: `draft`, `stable`, `deprecated`, or `archived`.

### 3. Invalid Share Enum
* **Severity**: Error (Level 1+)
* **Diagnostic Message**: `invalid share value: '<value>'`
* **Cause**: `share` in frontmatter is set to an unsupported value.
* **Resolution**: Change `share` to one of the three valid lowercase enums: `public`, `org`, or `private`.

### 4. Profile Expected Section Warning
* **Severity**: Warning (Level 3)
* **Diagnostic Message**: `Profile '<name>' expects heading '## <Section>', but it was missing`
* **Cause**: Document frontmatter declares `profile: <name>`, but the Markdown body lacks one or more `## H2` section headings specified by that profile.
* **Resolution**: Add the required `## <Section>` heading to your document, or switch to `profile: note` if the document has a custom structure.

### 5. Dangling `depends` or `related` Reference
* **Severity**: Error (Level 3)
* **Diagnostic Message**: `Dangling document reference: '<ref>' does not exist`
* **Cause**: A path listed in `depends:` or `related:` does not resolve to an existing document file.
* **Resolution**: Fix the path spelling, update the reference to point to the renamed document, or remove the entry. Run `ods fmt --refs md-paths` to auto-normalize Document refs to relative `.md` paths.

### 6. Duplicate Document ID
* **Severity**: Error (Level 3)
* **Diagnostic Message**: `Duplicate document ID found: '<id>' in '<path-a>' and '<path-b>'`
* **Cause**: Two distinct Markdown documents specify the exact same explicit `id:` in their frontmatter, or have conflicting path-derived IDs.
* **Resolution**: Remove the explicit `id:` override from one file (letting it use its relative path ID), or change the `id:` value to be unique.

### 7. Dependency Cycle Detected (`depends`)
* **Severity**: Error (Level 3)
* **Diagnostic Message**: `Dependency cycle detected: doc-a -> doc-b -> doc-a`
* **Cause**: Document A depends on Document B, which transitively depends back on Document A. `depends` forms a directed acyclic graph (DAG).
* **Resolution**: Remove the circular dependency. Move optional or bi-directional references from `depends:` into `related:`.

### 8. Missing Resource Path
* **Severity**: Error (Level 3)
* **Diagnostic Message**: `Resource path does not exist: '<path>'`
* **Cause**: A non-Markdown asset listed under `resources:` (`- path: assets/diagram.png`) does not exist on disk.
* **Resolution**: Check the file relative path and extension, move the asset to the expected location, or remove the `resources:` entry.

### 9. Missing Code Path or Invalid Role
* **Severity**: Error (Level 1+ for role, Level 3 for path)
* **Diagnostic Message**: `Invalid code role: '<role>'` or `Code reference path does not exist: '<path>'`
* **Cause**: A `code:` item specifies an unknown role or points to a source code file that does not exist.
* **Resolution**: Ensure `role` is one of the fixed roles (`entrypoint`, `implementation`, `test`, `schema`, `migration`, `config`, `infrastructure`, `pipeline`). Verify the target source file path.

### 10. Stale Index Bullet List
* **Severity**: Error (Level 3)
* **Diagnostic Message**: `Index '<path>/index.md' does not match immediate directory children`
* **Cause**: Files were added, deleted, or renamed without updating `index.md`.
* **Resolution**: Run `ods index` to regenerate all `index.md` child bullet lists lockfiles automatically.

### 11. Dangling Body Markdown Link
* **Severity**: Error (Level 3)
* **Diagnostic Message**: `Dangling body link '[Text]\(path/to/missing.md\)'`
* **Cause**: Standard Markdown link `[label]\(target.md\)` in prose points to a non-existent file or relative path.
* **Resolution**: Update the target link path or create the missing file.

---

## 3. Git Operations & Merge Conflict Resolution

### Merge Conflicts in `index.md` Lockfiles
Because `index.md` files act like lockfiles for directory listings, Git merges between active feature branches can occasionally produce merge conflicts in `index.md` child lists.

**Resolution**:
```bash
# Accept either side of the merge conflict in index.md
git checkout --ours **/index.md

# Regenerate exact indexes deterministically from the merged files
ods index

# Stage the resolved indexes
git add **/index.md
```

### Reconciling Git Renames (`ods sync`)
If files were renamed using standard `git mv` or an IDE refactoring tool while `ods serve` / `ods watch` was **not** running in the background:

```bash
ods sync
ods lint
```
