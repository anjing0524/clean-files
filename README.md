# 🧹 Clean Files

A fast, cross-platform command-line tool written in Rust to clean up development directories and free disk space.

## Features

- ✅ **Cross-platform**: Works on Windows, macOS, and Linux
- 🎯 **Multiple targets**: Clean Node.js, Rust, Python, and Java build artifacts
- 🔍 **Smart scanning**: Recursively finds and identifies cleanable directories
- 🛡️ **Safe**: Dry-run mode and confirmation prompts prevent accidents
- 📊 **Detailed stats**: Shows how much space you're freeing
- ⚡ **Blazing fast**: Parallel deletion with rayon for maximum performance
- 🎨 **Beautiful output**: Colored terminal output with progress bars
- 🔐 **Permission checks**: Verifies permissions before deletion to prevent errors

## Supported Directory Types

| Type | Directories | Description |
|------|------------|-------------|
| Node.js | `node_modules` | npm/yarn package directories |
| Rust | `target` | Cargo build artifacts |
| Python | `__pycache__`, `.pytest_cache`, `.tox`, `.mypy_cache` | Python bytecode and cache |
| Java | `target`, `build` | Maven and Gradle build directories |

## Installation

### From Source

```bash
git clone https://github.com/yourusername/clean-files.git
cd clean-files
cargo build --release
```

The binary will be available at `target/release/clean-files`.

### Using Cargo

```bash
cargo install --path .
```

## Usage

### Basic Usage

Clean all development directories in the current directory:

```bash
clean-files
```

Clean a specific directory:

```bash
clean-files /path/to/projects
```

### Options

```
USAGE:
    clean-files [OPTIONS] [PATH]

ARGS:
    <PATH>    Directory to scan (defaults to current directory)

OPTIONS:
    -t, --target <TARGET>      Type of directories to clean [default: all]
                               [possible values: node, rust, python, java, all]
    -n, --dry-run             Perform a dry run without actually deleting anything
    -v, --verbose             Show verbose output
    -d, --max-depth <DEPTH>   Maximum depth to scan (default: unlimited)
    -y, --yes                 Skip confirmation prompt (use with caution!)
    -j, --parallel            Use parallel processing for faster deletion [default: enabled]
    -h, --help                Print help information
    -V, --version             Print version information
```

### Examples

**Dry run to see what would be cleaned:**

```bash
clean-files --dry-run
```

**Clean only Node.js node_modules:**

```bash
clean-files --target node
```

**Clean only Rust target directories:**

```bash
clean-files --target rust
```

**Clean with verbose output:**

```bash
clean-files --verbose
```

**Clean without confirmation (careful!):**

```bash
clean-files --yes
```

**Limit scan depth:**

```bash
clean-files --max-depth 3
```

**Combine options:**

```bash
clean-files ~/projects --target all --dry-run --verbose
```

## How It Works

1. **Scan**: Recursively traverses the directory tree
2. **Identify**: Detects cleanable directories by checking for marker files:
   - `node_modules` → checks for `package.json` in parent
   - `target` → checks for `Cargo.toml` (Rust) or `pom.xml` (Java) in parent
   - `__pycache__` → Python bytecode cache
   - `build` → checks for `build.gradle` in parent
3. **Calculate**: Computes size and file count for each directory
4. **Confirm**: Shows summary and asks for confirmation (unless `--yes` or `--dry-run`)
5. **Clean**: Removes directories and shows statistics

## Safety Features

- **Dry run mode**: Test without deleting (`--dry-run`)
- **Confirmation prompt**: Asks before deleting (unless `--yes`)
- **Permission checks**: Verifies write permissions before attempting deletion
- **Marker verification**: Double-checks marker files exist before deletion
- **Smart detection**: Only removes directories with proper markers
- **Skip system dirs**: Ignores `.git`, `.svn`, etc.
- **Symlink safety**: Doesn't follow symbolic links
- **Error handling**: Continues on permission errors and reports failures
- **Race condition prevention**: Verifies directories still exist and match expected type

## Platform-Specific Considerations

### Windows

- Handles long path names (>260 characters)
- Automatically removes read-only attributes when needed
- Uses native path separators

### Linux/macOS

- Respects POSIX permissions
- Handles symbolic links properly
- Supports extended attributes

## Development

### Running Tests

Run all tests (unit and integration):

```bash
cargo test
```

Run with output:

```bash
cargo test -- --nocapture
```

Run specific test:

```bash
cargo test test_scanner_node_modules
```

### Test Coverage

The project includes:

- **Unit tests**: In each module (`types.rs`, `utils.rs`, `platform.rs`, etc.)
- **Integration tests**: In `tests/integration_test.rs`
- **Cross-platform tests**: Test platform-specific behavior

### Building for Release

```bash
cargo build --release
```

The optimized binary will be in `target/release/`.

For detailed cross-compilation instructions, see [BUILD.md](BUILD.md).

#### Supported Platforms

Pre-built binaries are available for:
- Linux x86_64 (glibc and musl)
- Linux ARM64 (glibc and musl)
- macOS x86_64 and ARM64 (Apple Silicon)
- Windows x86_64

See [BUILD.md](BUILD.md) for instructions on building for specific platforms.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [clap](https://github.com/clap-rs/clap) for CLI parsing
- Uses [walkdir](https://github.com/BurntSushi/walkdir) for directory traversal
- Parallel processing with [rayon](https://github.com/rayon-rs/rayon)
- Colored output with [colored](https://github.com/mackwic/colored)
- Progress bars with [indicatif](https://github.com/console-rs/indicatif)

## Safety Warning

⚠️ **Use with caution!** This tool permanently deletes files. Always:

- Use `--dry-run` first to preview what will be deleted
- Review the summary before confirming
- Keep backups of important data
- Avoid using `--yes` unless you're certain

## FAQ

**Q: Why do I need this tool?**

A: Development directories like `node_modules` and `target` can consume gigabytes of disk space. This tool helps you quickly identify and clean them across multiple projects.

**Q: Is it safe to delete these directories?**

A: Yes! These are build artifacts and can be regenerated:
- `node_modules` → `npm install` or `yarn install`
- `target` → `cargo build`
- `__pycache__` → regenerated automatically
- Java `target`/`build` → `mvn compile` or `gradle build`

**Q: Will this break my projects?**

A: No, but you'll need to rebuild/reinstall dependencies:
- Node.js: Run `npm install` or `yarn`
- Rust: Run `cargo build`
- Python: Caches regenerate automatically
- Java: Run your build tool (`mvn` or `gradle`)

**Q: Can I exclude certain directories?**

A: Currently not supported, but planned for future versions.

**Q: Does it work with monorepos?**

A: Yes! It recursively scans and finds all nested projects.

## Performance

`clean-files` is designed for speed:
- **Parallel deletion**: Uses rayon to delete multiple directories concurrently
- **Optimized traversal**: Efficient directory scanning with early pruning
- **Release optimizations**: LTO, single codegen unit, and stripped binaries
- **Smart skipping**: Avoids scanning inside target directories

Benchmark: Cleaning 1000+ node_modules directories with ~500K files typically completes in seconds.

## Roadmap

- [x] Parallel deletion for better performance
- [x] Multi-platform cross-compilation
- [x] Permission checks before deletion
- [ ] Configuration file support (`.cleanrc`)
- [ ] Exclude patterns
- [ ] More language support (Go, Swift, etc.)
- [ ] Interactive mode for selective cleaning
- [ ] Git-aware cleaning (skip uncommitted changes)
- [ ] Statistics history and tracking
