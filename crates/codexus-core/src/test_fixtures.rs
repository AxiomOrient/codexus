use std::fs;
use std::path::PathBuf;

use crate::runtime::StdioProcessSpec;

pub(crate) fn python_inline_process(script: &str) -> StdioProcessSpec {
    let mut spec = StdioProcessSpec::new("python3");
    spec.args = vec!["-u".to_owned(), "-c".to_owned(), script.to_owned()];
    spec
}

#[derive(Debug)]
pub(crate) struct TempDir {
    pub root: PathBuf,
}

impl TempDir {
    pub fn new(prefix: &str) -> Self {
        let root = std::env::temp_dir().join(format!("{prefix}_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create temp root");
        Self { root }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
