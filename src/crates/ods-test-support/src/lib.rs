use std::ffi::OsStr;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug)]
pub struct TempWorkspace(TempDir);

impl TempWorkspace {
    pub fn path(&self) -> &Path {
        self.0.path()
    }
}

impl Deref for TempWorkspace {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.0.path()
    }
}

impl AsRef<Path> for TempWorkspace {
    fn as_ref(&self) -> &Path {
        self.0.path()
    }
}

impl AsRef<OsStr> for TempWorkspace {
    fn as_ref(&self) -> &OsStr {
        self.0.path().as_os_str()
    }
}

pub fn temp_workspace() -> TempWorkspace {
    TempWorkspace(tempfile::tempdir().expect("temp workspace"))
}

pub fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/example-workspace")
}

pub fn copy_fixture_to_temp() -> TempWorkspace {
    let temp = tempfile::tempdir().expect("temp workspace");
    copy_dir_all(&fixture_root(), temp.path());
    TempWorkspace(temp)
}

fn copy_dir_all(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create fixture dir");
    for entry in fs::read_dir(from).expect("read fixture dir") {
        let entry = entry.expect("fixture entry");
        let path = entry.path();
        let target = to.join(entry.file_name());
        let file_type = entry.file_type().expect("fixture file type");
        if file_type.is_dir() {
            copy_dir_all(&path, &target);
        } else {
            fs::copy(&path, &target).expect("copy fixture file");
        }
    }
}
