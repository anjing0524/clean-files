use crate::platform::remove_dir_all;
use crate::types::{CleanStats, ScanResult};
use crate::utils::format_size;
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

pub struct Cleaner {
    dry_run: bool,
    verbose: bool,
}

impl Cleaner {
    pub fn new(dry_run: bool, verbose: bool) -> Self {
        Self { dry_run, verbose }
    }

    /// Clean the directories found by the scanner
    pub fn clean(&self, results: Vec<ScanResult>) -> Result<CleanStats> {
        self.clean_internal(results, true)
    }

    /// Internal clean method with optional confirmation
    fn clean_internal(&self, results: Vec<ScanResult>, require_confirmation: bool) -> Result<CleanStats> {
        let mut stats = CleanStats::default();

        if results.is_empty() {
            println!("{}", "No directories found to clean.".yellow());
            return Ok(stats);
        }

        // Show what will be cleaned
        self.print_summary(&results);

        // Ask for confirmation if not dry run
        if !self.dry_run && require_confirmation && !self.confirm_deletion() {
            println!("{}", "Cleanup cancelled.".yellow());
            return Ok(stats);
        }

        // Create progress bar
        let pb = if !self.dry_run && !self.verbose {
            let pb = ProgressBar::new(results.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            Some(pb)
        } else {
            None
        };

        // Process each result
        for result in results {
            if self.dry_run {
                // In dry-run mode, count everything as it would be deleted
                stats.add_result(&result);

                if self.verbose {
                    println!(
                        "{} {} {} ({})",
                        "[DRY RUN]".yellow(),
                        "Would delete:".white(),
                        result.path.display(),
                        format_size(result.size).cyan()
                    );
                }
            } else {
                if self.verbose {
                    println!(
                        "{} {}",
                        "Deleting:".red(),
                        result.path.display()
                    );
                }

                // Only add to stats if deletion succeeds
                match remove_dir_all(&result.path) {
                    Ok(_) => {
                        stats.add_result(&result);

                        if self.verbose {
                            println!(
                                "  {} {} freed",
                                "✓".green(),
                                format_size(result.size).cyan()
                            );
                        }
                    }
                    Err(e) => {
                        stats.add_failed();

                        eprintln!(
                            "{} Failed to delete {}: {}",
                            "✗".red(),
                            result.path.display(),
                            e
                        );
                    }
                }

                if let Some(ref pb) = pb {
                    pb.inc(1);
                }
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Done!");
        }

        Ok(stats)
    }

    /// Print a summary of what will be cleaned
    fn print_summary(&self, results: &[ScanResult]) {
        println!("\n{}", "=".repeat(60).cyan());
        if self.dry_run {
            println!("{}", "DRY RUN - No files will be deleted".yellow().bold());
        } else {
            println!("{}", "Cleanup Summary".cyan().bold());
        }
        println!("{}\n", "=".repeat(60).cyan());

        let total_size: u64 = results.iter().map(|r| r.size).sum();
        let total_files: usize = results.iter().map(|r| r.file_count).sum();

        println!("Found {} directories to clean:", results.len().to_string().green().bold());
        println!("Total size: {}", format_size(total_size).cyan().bold());
        println!("Total files: {}", total_files.to_string().yellow().bold());
        println!();

        if self.verbose {
            for result in results {
                println!(
                    "  {} {} {} ({}, {} files)",
                    "•".cyan(),
                    result.target_type.name().white().bold(),
                    result.path.display().to_string().dimmed(),
                    format_size(result.size).cyan(),
                    result.file_count.to_string().yellow()
                );
            }
            println!();
        }

        println!("{}", "=".repeat(60).cyan());
        println!();
    }

    /// Ask user for confirmation
    fn confirm_deletion(&self) -> bool {
        use std::io::{self, Write};

        print!("{}", "Do you want to proceed? [y/N]: ".yellow().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CleanTarget;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cleaner_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("to_clean");
        fs::create_dir(&test_dir).unwrap();
        fs::write(test_dir.join("file.txt"), "test content").unwrap();

        let mut result = ScanResult::new(test_dir.clone(), CleanTarget::NodeModules);
        result.size = 12;
        result.file_count = 1;

        let cleaner = Cleaner::new(true, false);
        let stats = cleaner.clean(vec![result]).unwrap();

        // Directory should still exist in dry run
        assert!(test_dir.exists());
        assert_eq!(stats.total_size, 12);
        assert_eq!(stats.total_files, 1);
    }

    #[test]
    fn test_cleaner_empty_results() {
        let cleaner = Cleaner::new(false, false);
        let stats = cleaner.clean(vec![]).unwrap();

        assert_eq!(stats.total_size, 0);
        assert_eq!(stats.total_files, 0);
    }

    #[test]
    fn test_cleaner_real_deletion() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("to_delete");
        fs::create_dir(&test_dir).unwrap();
        fs::write(test_dir.join("file1.txt"), "test content 1").unwrap();
        fs::write(test_dir.join("file2.txt"), "test content 2").unwrap();

        // Create nested directory
        let nested = test_dir.join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("file3.txt"), "nested content").unwrap();

        let mut result = ScanResult::new(test_dir.clone(), CleanTarget::NodeModules);
        result.size = 100;
        result.file_count = 3;

        // Real deletion (dry_run=false), skip confirmation for test
        let cleaner = Cleaner::new(false, false);
        let stats = cleaner.clean_internal(vec![result], false).unwrap();

        // Verify directory was actually deleted
        assert!(!test_dir.exists(), "Directory should be deleted");

        // Verify stats are correct
        assert_eq!(stats.total_size, 100);
        assert_eq!(stats.total_files, 3);
        assert_eq!(stats.total_dirs, 1);
        assert_eq!(stats.failed_dirs, 0);
        assert_eq!(stats.node_modules, 1);
    }

    #[test]
    fn test_cleaner_stats_accuracy() {
        // Test that stats correctly reflect multiple successful deletions
        let temp_dir = TempDir::new().unwrap();

        // Create first directory
        let dir1 = temp_dir.path().join("dir1");
        fs::create_dir(&dir1).unwrap();
        fs::write(dir1.join("file.txt"), "content1").unwrap();

        // Create second directory
        let dir2 = temp_dir.path().join("dir2");
        fs::create_dir(&dir2).unwrap();
        fs::write(dir2.join("file.txt"), "content2").unwrap();

        let mut result1 = ScanResult::new(dir1.clone(), CleanTarget::NodeModules);
        result1.size = 100;
        result1.file_count = 1;

        let mut result2 = ScanResult::new(dir2.clone(), CleanTarget::RustTarget);
        result2.size = 50;
        result2.file_count = 1;

        let cleaner = Cleaner::new(false, false);
        let stats = cleaner.clean_internal(vec![result1, result2], false).unwrap();

        // Both deletions should be counted
        assert_eq!(stats.total_dirs, 2, "Should count all successful deletions");
        assert_eq!(stats.total_size, 150, "Should sum all deleted sizes");
        assert_eq!(stats.total_files, 2, "Should count all files");
        assert_eq!(stats.failed_dirs, 0, "Should have no failures");
        assert_eq!(stats.node_modules, 1);
        assert_eq!(stats.rust_targets, 1);

        // Verify both dirs were deleted
        assert!(!dir1.exists());
        assert!(!dir2.exists());
    }
}
