/// Format bytes into human-readable size
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes = bytes as f64;
    let base = 1024_f64;
    let exponent = (bytes.ln() / base.ln()).floor() as usize;
    let exponent = exponent.min(UNITS.len() - 1);

    let value = bytes / base.powi(exponent as i32);
    let unit = UNITS[exponent];

    if exponent == 0 {
        format!("{} {}", value as u64, unit)
    } else {
        format!("{:.2} {}", value, unit)
    }
}

/// Check if a directory name should be skipped during scanning
pub fn should_skip_directory(dir_name: &str) -> bool {
    matches!(
        dir_name,
        ".git" | ".svn" | ".hg" | ".bzr" | ".darcs" | "node_modules" | "target" | "__pycache__" | "build"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1536 * 1024), "1.50 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1024_u64.pow(4)), "1.00 TB");
    }

    #[test]
    fn test_should_skip_directory() {
        assert!(should_skip_directory(".git"));
        assert!(should_skip_directory("node_modules"));
        assert!(should_skip_directory("target"));
        assert!(should_skip_directory("__pycache__"));
        assert!(!should_skip_directory("src"));
        assert!(!should_skip_directory("test"));
    }
}
