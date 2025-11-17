use crate::platform::calculate_dir_size;
use crate::types::{CleanTarget, ScanResult};
use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

pub struct Scanner {
    target: CleanTarget,
    max_depth: Option<usize>,
    verbose: bool,
}

impl Scanner {
    pub fn new(target: CleanTarget) -> Self {
        Self {
            target,
            max_depth: None,
            verbose: false,
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Scan a directory for cleanable targets
    pub fn scan(&self, root: &Path) -> Result<Vec<ScanResult>> {
        let mut results = Vec::new();

        let mut walker = if let Some(depth) = self.max_depth {
            WalkDir::new(root).max_depth(depth)
        } else {
            WalkDir::new(root)
        };

        walker = walker.min_depth(1);

        for entry in walker.into_iter().filter_entry(|e| self.should_enter(e)) {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    // Log permission errors or other access issues
                    if self.verbose {
                        eprintln!("⚠️  Skipped (access error): {}", e);
                    }
                    continue;
                }
            };

            if !entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path();
            let dir_name = match path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => continue,
            };

            // Check if this directory matches any of our targets
            if let Some(target_type) = self.identify_target(&dir_name, path) {
                if self.target.should_clean(&target_type) {
                    let mut result = ScanResult::new(path.to_path_buf(), target_type);

                    // Calculate size and file count
                    if let Ok((size, count)) = calculate_dir_size(path) {
                        result.size = size;
                        result.file_count = count;
                    }

                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    /// Determine if we should enter a directory during traversal
    fn should_enter(&self, entry: &walkdir::DirEntry) -> bool {
        if !entry.file_type().is_dir() {
            return true;
        }

        let dir_name = entry.file_name().to_string_lossy();

        // Don't enter version control directories
        if matches!(
            dir_name.as_ref(),
            ".git" | ".svn" | ".hg" | ".bzr" | ".darcs"
        ) {
            return false;
        }

        // Check if parent directory is one of our target types
        // If so, don't descend (we'll process the parent as a target)
        if let Some(parent) = entry.path().parent() {
            if let Some(parent_name) = parent.file_name() {
                let parent_name = parent_name.to_string_lossy();
                // Don't descend into contents of target directories
                if matches!(
                    parent_name.as_ref(),
                    "node_modules"
                        | "target"
                        | "__pycache__"
                        | "build"
                        | ".pytest_cache"
                        | ".tox"
                        | ".mypy_cache"
                ) {
                    return false;
                }
            }
        }

        true
    }

    /// Identify what type of cleanable directory this is
    fn identify_target(&self, dir_name: &str, path: &Path) -> Option<CleanTarget> {
        match dir_name {
            "node_modules" => {
                // Verify it's a node_modules by checking for package.json in parent
                if let Some(parent) = path.parent() {
                    if parent.join("package.json").exists() {
                        return Some(CleanTarget::NodeModules);
                    }
                }
                // Also accept it if it looks like node_modules
                Some(CleanTarget::NodeModules)
            }
            "target" => {
                // Check if it's a Rust target (has Cargo.toml in parent)
                if let Some(parent) = path.parent() {
                    if parent.join("Cargo.toml").exists() {
                        return Some(CleanTarget::RustTarget);
                    }
                    // Check if it's a Maven/Gradle target (has pom.xml or build.gradle)
                    if parent.join("pom.xml").exists()
                        || parent.join("build.gradle").exists()
                        || parent.join("build.gradle.kts").exists()
                    {
                        return Some(CleanTarget::JavaTarget);
                    }
                }
                None
            }
            "build" => {
                // Gradle build directory
                if let Some(parent) = path.parent() {
                    if parent.join("build.gradle").exists()
                        || parent.join("build.gradle.kts").exists()
                    {
                        return Some(CleanTarget::JavaTarget);
                    }
                }
                None
            }
            "__pycache__" => Some(CleanTarget::PythonCache),
            ".pytest_cache" | ".tox" | ".mypy_cache" => Some(CleanTarget::PythonCache),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scanner_node_modules() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("myproject");
        fs::create_dir(&project_dir).unwrap();

        // Create package.json and node_modules
        fs::write(project_dir.join("package.json"), "{}").unwrap();
        let node_modules = project_dir.join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        fs::write(node_modules.join("test.js"), "console.log('test');").unwrap();

        let scanner = Scanner::new(CleanTarget::NodeModules);
        let results = scanner.scan(&project_dir).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].target_type, CleanTarget::NodeModules);
        assert!(results[0].size > 0);
    }

    #[test]
    fn test_scanner_rust_target() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("rust-project");
        fs::create_dir(&project_dir).unwrap();

        // Create Cargo.toml and target
        fs::write(project_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        let target = project_dir.join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("test.rs"), "fn main() {}").unwrap();

        let scanner = Scanner::new(CleanTarget::RustTarget);
        let results = scanner.scan(&project_dir).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].target_type, CleanTarget::RustTarget);
    }

    #[test]
    fn test_scanner_python_cache() {
        let temp_dir = TempDir::new().unwrap();
        let pycache = temp_dir.path().join("__pycache__");
        fs::create_dir(&pycache).unwrap();
        fs::write(pycache.join("test.pyc"), &[0u8; 100]).unwrap();

        let scanner = Scanner::new(CleanTarget::PythonCache);
        let results = scanner.scan(temp_dir.path()).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].target_type, CleanTarget::PythonCache);
    }

    #[test]
    fn test_scanner_all_targets() {
        let temp_dir = TempDir::new().unwrap();

        // Create node_modules
        let node_project = temp_dir.path().join("node-app");
        fs::create_dir(&node_project).unwrap();
        fs::write(node_project.join("package.json"), "{}").unwrap();
        fs::create_dir(node_project.join("node_modules")).unwrap();

        // Create Python cache
        fs::create_dir(temp_dir.path().join("__pycache__")).unwrap();

        let scanner = Scanner::new(CleanTarget::All);
        let results = scanner.scan(temp_dir.path()).unwrap();

        assert!(results.len() >= 2);
    }
}
