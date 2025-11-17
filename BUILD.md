# Building Clean Files

This document describes how to build `clean-files` for different platforms.

## Prerequisites

- Rust 1.70 or later (install from https://rustup.rs/)
- A C compiler (GCC, Clang, or MSVC depending on your platform)

## Quick Start

### Build for your current platform

```bash
cargo build --release
```

The binary will be at `target/release/clean-files` (or `clean-files.exe` on Windows).

### Run tests

```bash
cargo test
```

## Cross-Compilation

### Linux Targets

#### x86_64 Linux (glibc)
```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

#### x86_64 Linux (musl - static binary)
```bash
# Install musl tools
sudo apt-get install musl-tools

# Add target
rustup target add x86_64-unknown-linux-musl

# Build
cargo build --release --target x86_64-unknown-linux-musl
```

#### ARM64 Linux (glibc)
```bash
# Install cross-compiler
sudo apt-get install gcc-aarch64-linux-gnu

# Add target
rustup target add aarch64-unknown-linux-gnu

# Build
cargo build --release --target aarch64-unknown-linux-gnu
```

#### ARM64 Linux (musl - static binary)
```bash
# Install tools
sudo apt-get install gcc-aarch64-linux-gnu musl-tools

# Add target
rustup target add aarch64-unknown-linux-musl

# Build
cargo build --release --target aarch64-unknown-linux-musl
```

### macOS Targets

#### x86_64 macOS
```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

#### ARM64 macOS (Apple Silicon)
```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

### Windows Targets

#### x86_64 Windows (MSVC)
```bash
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

#### x86_64 Windows (GNU)
```bash
# On Linux, install MinGW
sudo apt-get install mingw-w64

# Add target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

## Using cargo-cross

For easier cross-compilation, you can use `cargo-cross`:

```bash
# Install cross
cargo install cross

# Build for any target
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-pc-windows-gnu
```

## Build Profiles

### Development Build
```bash
cargo build
```
- Fast compilation
- Includes debug symbols
- No optimizations

### Release Build
```bash
cargo build --release
```
- Optimized for speed and size
- LTO (Link Time Optimization) enabled
- Debug symbols stripped
- Panic strategy set to abort

## Performance Optimization Features

The release build includes:
- **Parallel deletion**: Uses rayon for concurrent directory deletion
- **LTO**: Link-time optimization for smaller binaries
- **Codegen units = 1**: Single codegen unit for maximum optimization
- **Strip = true**: Debug symbols removed for smaller size

## Platform-Specific Notes

### Linux
- For maximum portability, use musl targets (static linking)
- glibc targets produce smaller binaries but require compatible glibc version

### macOS
- Universal binaries can be created by combining x86_64 and aarch64 builds:
  ```bash
  lipo -create \
    target/x86_64-apple-darwin/release/clean-files \
    target/aarch64-apple-darwin/release/clean-files \
    -output clean-files-universal
  ```

### Windows
- MSVC target is recommended for native Windows development
- GNU target is useful for cross-compilation from Linux

## CI/CD

The project includes GitHub Actions workflows:
- **CI**: Tests on Linux, macOS, and Windows
- **Release**: Builds binaries for all platforms on tagged releases

## Troubleshooting

### Linker not found
Make sure you have the appropriate cross-compilation tools installed for your target platform.

### Permission errors on Linux/macOS
Ensure the binary has execute permissions:
```bash
chmod +x target/release/clean-files
```

### Windows Defender false positives
Some antivirus software may flag the binary. This is a false positive due to the aggressive optimizations.
