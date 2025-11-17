mod cleaner;
mod cli;
mod platform;
mod scanner;
mod types;
mod utils;

use anyhow::Result;
use clap::Parser;
use cleaner::Cleaner;
use cli::Cli;
use colored::*;
use scanner::Scanner;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use types::CleanTarget;
use utils::format_size;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up Ctrl+C handler for graceful shutdown
    let interrupted = Arc::new(AtomicBool::new(false));
    let r = interrupted.clone();

    ctrlc::set_handler(move || {
        r.store(true, Ordering::SeqCst);
        eprintln!(
            "\n{}",
            "⚠️  Interrupt received! Stopping cleanup gracefully..."
                .yellow()
                .bold()
        );
    })?;

    // Print banner
    print_banner();

    // Validate path
    if !cli.path.exists() {
        eprintln!(
            "{} Path does not exist: {}",
            "Error:".red().bold(),
            cli.path.display()
        );
        std::process::exit(1);
    }

    if !cli.path.is_dir() {
        eprintln!(
            "{} Path is not a directory: {}",
            "Error:".red().bold(),
            cli.path.display()
        );
        std::process::exit(1);
    }

    // Convert target type
    let target: CleanTarget = cli.target.into();

    println!(
        "Scanning directory: {}",
        cli.path.display().to_string().cyan().bold()
    );
    println!("Target: {}", target.name().green());
    if cli.dry_run {
        println!(
            "{}",
            "Mode: DRY RUN (no files will be deleted)".yellow().bold()
        );
    }
    println!();

    // Scan for targets
    println!("{}", "Scanning...".yellow());
    let mut scanner = Scanner::new(target);
    if let Some(depth) = cli.max_depth {
        scanner = scanner.with_max_depth(depth);
    }
    if cli.verbose {
        scanner = scanner.with_verbose(true);
    }

    let results = scanner.scan(&cli.path)?;

    // Clean the targets
    let cleaner = Cleaner::new(cli.dry_run, cli.verbose)
        .with_interrupt_flag(interrupted)
        .with_parallel(cli.parallel);

    // Override confirmation if --yes flag is set
    let stats = if cli.yes && !cli.dry_run {
        println!("{}", "Skipping confirmation (--yes flag set)".yellow());
        cleaner.clean_without_confirmation(results)?
    } else {
        cleaner.clean(results)?
    };

    // Print final statistics
    print_stats(&stats, cli.dry_run);

    Ok(())
}

fn print_banner() {
    let banner = r#"
╔═══════════════════════════════════════════════════════════╗
║              🧹 Clean Files - Dev Cleanup Tool            ║
║            Clear space by removing build artifacts        ║
╚═══════════════════════════════════════════════════════════╝
"#;
    println!("{}", banner.cyan());
}

fn print_stats(stats: &types::CleanStats, dry_run: bool) {
    println!("\n{}", "=".repeat(60).cyan());
    if dry_run {
        println!(
            "{}",
            "DRY RUN - Statistics (no files were deleted)"
                .yellow()
                .bold()
        );
    } else {
        println!("{}", "Cleanup Complete!".green().bold());
    }
    println!("{}", "=".repeat(60).cyan());
    println!();

    if stats.total_dirs == 0 {
        println!("{}", "No directories were found to clean.".yellow());
        return;
    }

    println!("📊 Statistics:");
    println!(
        "  • Total directories cleaned: {}",
        stats.total_dirs.to_string().green().bold()
    );
    println!(
        "  • Total space freed: {}",
        format_size(stats.total_size).cyan().bold()
    );
    println!(
        "  • Total files removed: {}",
        stats.total_files.to_string().yellow().bold()
    );

    if !dry_run && (stats.failed_dirs > 0 || stats.skipped_dirs > 0) {
        println!();
        println!("⚠️  Errors & Warnings:");
        if stats.failed_dirs > 0 {
            println!(
                "  • Failed to delete: {}",
                stats.failed_dirs.to_string().red().bold()
            );
        }
        if stats.skipped_dirs > 0 {
            println!(
                "  • Skipped (interrupted): {}",
                stats.skipped_dirs.to_string().yellow().bold()
            );
        }
    }

    println!();

    println!("🗂️  Breakdown by type:");
    if stats.node_modules > 0 {
        println!(
            "  • Node.js (node_modules): {}",
            stats.node_modules.to_string().green()
        );
    }
    if stats.rust_targets > 0 {
        println!(
            "  • Rust (target): {}",
            stats.rust_targets.to_string().green()
        );
    }
    if stats.python_caches > 0 {
        println!(
            "  • Python (__pycache__): {}",
            stats.python_caches.to_string().green()
        );
    }
    if stats.java_targets > 0 {
        println!(
            "  • Java (target/build): {}",
            stats.java_targets.to_string().green()
        );
    }

    println!();
    println!("{}", "=".repeat(60).cyan());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
    }
}
