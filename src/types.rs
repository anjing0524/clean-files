use std::path::PathBuf;

/// Types of directories that can be cleaned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanTarget {
    NodeModules,
    RustTarget,
    PythonCache,
    JavaTarget,
    All,
}

impl CleanTarget {
    /// Returns all available clean target types (excluding All)
    /// This is useful for programmatic iteration and testing
    #[allow(dead_code)]
    pub fn all_targets() -> Vec<CleanTarget> {
        vec![
            CleanTarget::NodeModules,
            CleanTarget::RustTarget,
            CleanTarget::PythonCache,
            CleanTarget::JavaTarget,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            CleanTarget::NodeModules => "node_modules",
            CleanTarget::RustTarget => "rust target",
            CleanTarget::PythonCache => "python __pycache__",
            CleanTarget::JavaTarget => "java target/build",
            CleanTarget::All => "all",
        }
    }

    pub fn should_clean(&self, other: &CleanTarget) -> bool {
        self == &CleanTarget::All || self == other
    }
}

/// Result of scanning a directory
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub path: PathBuf,
    pub target_type: CleanTarget,
    pub size: u64,
    pub file_count: usize,
}

impl ScanResult {
    pub fn new(path: PathBuf, target_type: CleanTarget) -> Self {
        Self {
            path,
            target_type,
            size: 0,
            file_count: 0,
        }
    }
}

/// Statistics for the cleanup operation
#[derive(Debug, Default)]
pub struct CleanStats {
    pub total_size: u64,
    pub total_files: usize,
    pub total_dirs: usize,
    pub node_modules: usize,
    pub rust_targets: usize,
    pub python_caches: usize,
    pub java_targets: usize,
    pub failed_dirs: usize,
    pub skipped_dirs: usize,
}

impl CleanStats {
    pub fn add_result(&mut self, result: &ScanResult) {
        self.total_size += result.size;
        self.total_files += result.file_count;
        self.total_dirs += 1;

        match result.target_type {
            CleanTarget::NodeModules => self.node_modules += 1,
            CleanTarget::RustTarget => self.rust_targets += 1,
            CleanTarget::PythonCache => self.python_caches += 1,
            CleanTarget::JavaTarget => self.java_targets += 1,
            CleanTarget::All => {}
        }
    }

    pub fn add_failed(&mut self) {
        self.failed_dirs += 1;
    }

    pub fn add_skipped(&mut self) {
        self.skipped_dirs += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_target_all_targets() {
        let targets = CleanTarget::all_targets();
        assert_eq!(targets.len(), 4);
        assert!(targets.contains(&CleanTarget::NodeModules));
        assert!(targets.contains(&CleanTarget::RustTarget));
        assert!(targets.contains(&CleanTarget::PythonCache));
        assert!(targets.contains(&CleanTarget::JavaTarget));
    }

    #[test]
    fn test_clean_target_should_clean() {
        assert!(CleanTarget::All.should_clean(&CleanTarget::NodeModules));
        assert!(CleanTarget::NodeModules.should_clean(&CleanTarget::NodeModules));
        assert!(!CleanTarget::NodeModules.should_clean(&CleanTarget::RustTarget));
    }

    #[test]
    fn test_clean_stats() {
        let mut stats = CleanStats::default();
        let result = ScanResult {
            path: PathBuf::from("/test"),
            target_type: CleanTarget::NodeModules,
            size: 1024,
            file_count: 10,
        };

        stats.add_result(&result);

        assert_eq!(stats.total_size, 1024);
        assert_eq!(stats.total_files, 10);
        assert_eq!(stats.total_dirs, 1);
        assert_eq!(stats.node_modules, 1);
    }
}
