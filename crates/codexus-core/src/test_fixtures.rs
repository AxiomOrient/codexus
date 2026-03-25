use std::fs;
use std::panic::Location;
use std::path::{Path, PathBuf};

use crate::runtime::StdioProcessSpec;
use sha2::{Digest, Sha256};

pub(crate) fn python_inline_process(script: &str) -> StdioProcessSpec {
    let mut spec = StdioProcessSpec::new("python3");
    spec.args = vec!["-u".to_owned(), "-c".to_owned(), script.to_owned()];
    spec
}

pub(crate) fn write_executable_script(path: &Path, script: &str) {
    fs::write(path, script).expect("write script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path).expect("script metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("set script executable");
    }
}

#[derive(Debug)]
pub(crate) struct TempDir {
    pub root: PathBuf,
}

impl TempDir {
    #[track_caller]
    pub fn new(prefix: &str) -> Self {
        let root = deterministic_temp_path(prefix, Location::caller());
        fs::create_dir_all(&root).expect("create temp root");
        Self { root }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn deterministic_temp_path(prefix: &str, location: &Location<'_>) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(location.file().as_bytes());
    hasher.update(location.line().to_le_bytes());
    hasher.update(location.column().to_le_bytes());
    // Include thread ID so that parallel tests on different threads get different paths
    // and cannot race on the same directory. Within a single thread, paths remain
    // stable across sequential test runs (same thread → same path, cleaned up by Drop).
    hasher.update(format!("{:?}", std::thread::current().id()).as_bytes());
    let digest = hex::encode(hasher.finalize());
    std::env::temp_dir().join(format!("{prefix}_{}", &digest[..16]))
}
