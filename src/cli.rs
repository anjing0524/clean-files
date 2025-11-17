use crate::types::CleanTarget;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "clean-files")]
#[command(author = "Clean Files Contributors")]
#[command(version = "0.1.0")]
#[command(about = "Clean development directories (node_modules, target, __pycache__, etc.)", long_about = None)]
pub struct Cli {
    /// Directory to scan (defaults to current directory)
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Type of directories to clean
    #[arg(short, long, value_enum, default_value = "all")]
    pub target: TargetType,

    /// Perform a dry run without actually deleting anything
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Maximum depth to scan (default: unlimited)
    #[arg(short = 'd', long)]
    pub max_depth: Option<usize>,

    /// Skip confirmation prompt (use with caution!)
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Use parallel processing for faster deletion (default: enabled)
    #[arg(short = 'j', long = "parallel", default_value = "true")]
    pub parallel: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TargetType {
    /// Node.js node_modules directories
    Node,
    /// Rust target directories
    Rust,
    /// Python __pycache__ directories
    Python,
    /// Java/Maven/Gradle target/build directories
    Java,
    /// All supported directory types
    All,
}

impl From<TargetType> for CleanTarget {
    fn from(target: TargetType) -> Self {
        match target {
            TargetType::Node => CleanTarget::NodeModules,
            TargetType::Rust => CleanTarget::RustTarget,
            TargetType::Python => CleanTarget::PythonCache,
            TargetType::Java => CleanTarget::JavaTarget,
            TargetType::All => CleanTarget::All,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_type_conversion() {
        assert_eq!(
            CleanTarget::from(TargetType::Node),
            CleanTarget::NodeModules
        );
        assert_eq!(CleanTarget::from(TargetType::Rust), CleanTarget::RustTarget);
        assert_eq!(
            CleanTarget::from(TargetType::Python),
            CleanTarget::PythonCache
        );
        assert_eq!(CleanTarget::from(TargetType::Java), CleanTarget::JavaTarget);
        assert_eq!(CleanTarget::from(TargetType::All), CleanTarget::All);
    }
}
