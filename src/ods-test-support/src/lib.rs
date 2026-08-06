use std::ffi::OsStr;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::Duration;
use tempfile::TempDir;

/// RAII guard that **always** terminates a spawned child on drop (SIGTERM then kill).
///
/// Use for `ods serve` / `ods watch` integration tests so processes cannot leak
/// after panics or early returns.
pub struct ChildGuard {
    child: Option<Child>,
}

impl ChildGuard {
    pub fn new(child: Child) -> Self {
        Self { child: Some(child) }
    }

    pub fn child_mut(&mut self) -> Option<&mut Child> {
        self.child.as_mut()
    }

    /// Graceful terminate (Unix SIGTERM) then hard kill; wait for exit.
    pub fn terminate(mut self) -> std::io::Result<std::process::ExitStatus> {
        if let Some(mut child) = self.child.take() {
            terminate_child(&mut child);
            child.wait()
        } else {
            Err(std::io::Error::other("child already taken"))
        }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            terminate_child(&mut child);
            let _ = child.wait();
        }
    }
}

impl Deref for ChildGuard {
    type Target = Child;
    fn deref(&self) -> &Self::Target {
        self.child.as_ref().expect("ChildGuard empty")
    }
}

impl DerefMut for ChildGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.child.as_mut().expect("ChildGuard empty")
    }
}

fn terminate_child(child: &mut Child) {
    #[cfg(unix)]
    {
        // SAFETY: signal delivery only; valid pid from Child::id().
        unsafe {
            libc::kill(child.id() as libc::pid_t, libc::SIGTERM);
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    let _ = child.kill();
}

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

/// Write a minimal `ods.toml` workspace marker (spec 0.1).
pub fn write_ods_toml(root: &Path) {
    write_ods_toml_with(root, "spec = \"0.1\"\n");
}

/// Write `ods.toml` with custom body.
pub fn write_ods_toml_with(root: &Path, body: &str) {
    fs::write(root.join("ods.toml"), body).expect("write ods.toml");
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
