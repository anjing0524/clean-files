use crate::platform::remove_dir_all;
use crate::types::{CleanStats, ScanResult};
use crate::utils::format_size;
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct Cleaner {
    dry_run: bool,
    verbose: bool,
    interrupt_flag: Option<Arc<AtomicBool>>,
    parallel: bool,
}

impl Cleaner {
    pub fn new(dry_run: bool, verbose: bool) -> Self {
        Self {
            dry_run,
            verbose,
            interrupt_flag: None,
            parallel: true, // Enable parallel processing by default
        }
    }

    /// Enable or disable parallel processing
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Add an interrupt flag for graceful shutdown on Ctrl+C
    pub fn with_interrupt_flag(mut self, flag: Arc<AtomicBool>) -> Self {
        self.interrupt_flag = Some(flag);
        self
    }

    /// Check if the operation has been interrupted
    fn is_interrupted(&self) -> bool {
        self.interrupt_flag
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::SeqCst))
    }

    /// Verify directory before deletion to prevent race conditions
    fn verify_before_delete(&self, result: &ScanResult) -> Result<(), String> {
        use crate::platform::can_delete;
        use crate::types::CleanTarget;

        // Check if directory still exists
        if !result.path.exists() {
            return Err(format!(
                "Directory no longer exists: {}",
                result.path.display()
            ));
        }

        // Check if it's still a directory
        if !result.path.is_dir() {
            return Err(format!(
                "Path is no longer a directory: {}",
                result.path.display()
            ));
        }

        // Check if we have permission to delete
        if !can_delete(&result.path) {
            return Err(format!(
                "Permission denied or directory is read-only: {}",
                result.path.display()
            ));
        }

        // Verify marker files based on target type
        let parent = match result.path.parent() {
            Some(p) => p,
            None => return Ok(()), // Root-level directory, skip marker check
        };

        let verified = match result.target_type {
            CleanTarget::NodeModules => {
                // Verify package.json exists for node_modules
                parent.join("package.json").exists()
            }
            CleanTarget::RustTarget => {
                // Verify Cargo.toml exists for Rust target
                parent.join("Cargo.toml").exists()
            }
            CleanTarget::JavaTarget => {
                // Verify pom.xml or build.gradle exists for Java targets
                parent.join("pom.xml").exists()
                    || parent.join("build.gradle").exists()
                    || parent.join("build.gradle.kts").exists()
            }
            CleanTarget::PythonCache => {
                // Python cache doesn't require marker file verification
                true
            }
            CleanTarget::All => true,
        };

        if !verified {
            return Err(format!(
                "Marker file verification failed for {}: {}",
                result.target_type.name(),
                result.path.display()
            ));
        }

        Ok(())
    }

    /// Clean the directories found by the scanner
    pub fn clean(&self, results: Vec<ScanResult>) -> Result<CleanStats> {
        self.clean_internal(results, true)
    }

    /// Clean without confirmation (for --yes flag)
    pub fn clean_without_confirmation(&self, results: Vec<ScanResult>) -> Result<CleanStats> {
        self.clean_internal(results, false)
    }

    /// Internal clean method with optional confirmation
    fn clean_internal(
        &self,
        results: Vec<ScanResult>,
        require_confirmation: bool,
    ) -> Result<CleanStats> {
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

        // Get total count for progress bar
        let total = results.len();

        // Create progress bar (for both dry-run and real mode if not verbose)
        let pb = if !self.verbose {
            let pb = ProgressBar::new(total as u64);
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

        // Process results - use parallel processing if enabled and not in verbose mode
        if self.parallel && !self.verbose && results.len() > 1 {
            // Parallel processing for better performance with many directories
            self.process_parallel(results, &pb, &mut stats)?;
        } else {
            // Sequential processing for verbose mode or single directory
            self.process_sequential(results, &pb, &mut stats)?;
        }

        // Finish progress bar with appropriate message
        if let Some(pb) = pb {
            if self.dry_run {
                pb.finish_with_message("Dry run complete!");
            } else if stats.failed_dirs > 0 || stats.skipped_dirs > 0 {
                pb.finish_with_message(format!(
                    "Done with {} warnings",
                    stats.failed_dirs + stats.skipped_dirs
                ));
            } else {
                pb.finish_with_message("Successfully completed!");
            }
        }

        Ok(stats)
    }

    /// Process results sequentially (for verbose mode or when parallel is disabled)
    fn process_sequential(
        &self,
        results: Vec<ScanResult>,
        pb: &Option<ProgressBar>,
        stats: &mut CleanStats,
    ) -> Result<()> {
        let total = results.len();

        for (idx, result) in results.into_iter().enumerate() {
            // Check for interruption
            if self.is_interrupted() {
                // Count remaining directories as skipped
                let remaining = total - idx;
                for _ in 0..remaining {
                    stats.add_skipped();
                }

                if let Some(ref pb) = pb {
                    pb.finish_with_message("Interrupted!");
                }

                println!(
                    "\n{}",
                    "⚠️  Cleanup interrupted by user. Remaining directories skipped."
                        .yellow()
                        .bold()
                );
                break;
            }

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
                } else if let Some(ref pb) = pb {
                    let dir_name = result
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    pb.set_message(format!("Checking: {}", dir_name));
                    pb.inc(1);
                }
            } else {
                // Verify before deletion to prevent race conditions
                if let Err(e) = self.verify_before_delete(&result) {
                    stats.add_skipped();

                    if self.verbose {
                        eprintln!("{} Skipped {}: {}", "⚠️".yellow(), result.path.display(), e);
                    }

                    if let Some(ref pb) = pb {
                        pb.inc(1);
                    }
                    continue;
                }

                if self.verbose {
                    println!("{} {}", "Deleting:".red(), result.path.display());
                }

                // Update progress bar with current directory name
                if let Some(ref pb) = pb {
                    let dir_name = result
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    pb.set_message(format!("Deleting: {}", dir_name));
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

                        if let Some(ref pb) = pb {
                            pb.inc(1);
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

                        if let Some(ref pb) = pb {
                            pb.inc(1);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Process results in parallel for better performance
    fn process_parallel(
        &self,
        results: Vec<ScanResult>,
        pb: &Option<ProgressBar>,
        stats: &mut CleanStats,
    ) -> Result<()> {
        let stats_mutex = Arc::new(Mutex::new(CleanStats::default()));
        let pb_arc = pb.as_ref().map(|p| Arc::new(p.clone()));
        let processed = Arc::new(AtomicUsize::new(0));
        let total = results.len();

        // Process in parallel using rayon
        results.par_iter().try_for_each(|result| -> Result<()> {
            // Check for interruption
            if self.is_interrupted() {
                return Ok(());
            }

            if self.dry_run {
                // In dry-run mode, count everything as it would be deleted
                let mut stats_guard = stats_mutex.lock().unwrap();
                stats_guard.add_result(result);

                if let Some(ref pb) = pb_arc {
                    let dir_name = result
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    pb.set_message(format!("Checking: {}", dir_name));
                    pb.inc(1);
                }
            } else {
                // Verify before deletion to prevent race conditions
                if let Err(_e) = self.verify_before_delete(result) {
                    let mut stats_guard = stats_mutex.lock().unwrap();
                    stats_guard.add_skipped();

                    if let Some(ref pb) = pb_arc {
                        pb.inc(1);
                    }
                    return Ok(());
                }

                // Update progress bar with current directory name
                if let Some(ref pb) = pb_arc {
                    let dir_name = result
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    pb.set_message(format!("Deleting: {}", dir_name));
                }

                // Only add to stats if deletion succeeds
                match remove_dir_all(&result.path) {
                    Ok(_) => {
                        let mut stats_guard = stats_mutex.lock().unwrap();
                        stats_guard.add_result(result);

                        if let Some(ref pb) = pb_arc {
                            pb.inc(1);
                        }
                    }
                    Err(e) => {
                        let mut stats_guard = stats_mutex.lock().unwrap();
                        stats_guard.add_failed();

                        eprintln!(
                            "{} Failed to delete {}: {}",
                            "✗".red(),
                            result.path.display(),
                            e
                        );

                        if let Some(ref pb) = pb_arc {
                            pb.inc(1);
                        }
                    }
                }
            }

            processed.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })?;

        // Count skipped directories if interrupted
        let processed_count = processed.load(Ordering::SeqCst);
        if processed_count < total {
            let mut stats_guard = stats_mutex.lock().unwrap();
            for _ in 0..(total - processed_count) {
                stats_guard.add_skipped();
            }

            if let Some(ref pb) = pb {
                pb.finish_with_message("Interrupted!");
            }

            println!(
                "\n{}",
                "⚠️  Cleanup interrupted by user. Remaining directories skipped."
                    .yellow()
                    .bold()
            );
        }

        // Merge parallel stats back into main stats
        let final_stats = Arc::try_unwrap(stats_mutex).unwrap().into_inner().unwrap();
        *stats = final_stats;

        Ok(())
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

        println!(
            "Found {} directories to clean:",
            results.len().to_string().green().bold()
        );
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
        let project_dir = temp_dir.path().join("myproject");
        fs::create_dir(&project_dir).unwrap();

        // Create package.json for node_modules verification
        fs::write(project_dir.join("package.json"), "{}").unwrap();

        let test_dir = project_dir.join("node_modules");
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

        // Create first project with node_modules
        let project1 = temp_dir.path().join("project1");
        fs::create_dir(&project1).unwrap();
        fs::write(project1.join("package.json"), "{}").unwrap();

        let dir1 = project1.join("node_modules");
        fs::create_dir(&dir1).unwrap();
        fs::write(dir1.join("file.txt"), "content1").unwrap();

        // Create second project with Rust target
        let project2 = temp_dir.path().join("project2");
        fs::create_dir(&project2).unwrap();
        fs::write(project2.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let dir2 = project2.join("target");
        fs::create_dir(&dir2).unwrap();
        fs::write(dir2.join("file.txt"), "content2").unwrap();

        let mut result1 = ScanResult::new(dir1.clone(), CleanTarget::NodeModules);
        result1.size = 100;
        result1.file_count = 1;

        let mut result2 = ScanResult::new(dir2.clone(), CleanTarget::RustTarget);
        result2.size = 50;
        result2.file_count = 1;

        let cleaner = Cleaner::new(false, false);
        let stats = cleaner
            .clean_internal(vec![result1, result2], false)
            .unwrap();

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
