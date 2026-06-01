# Contributing to Clipboard GUI

Thank you for your interest in contributing!

## Development Setup

```bash
# Clone the repository
git clone https://github.com/CREATSAIF/clipboard-gui.git
cd clipboard-gui

# Build in development mode
cargo build --verbose

# Run the application
cargo run
```

## Project Structure

```
clipboard-gui/
├── src/
│   └── main.rs          # Main application code
├── Cargo.toml            # Rust dependencies
├── Cargo.lock            # Dependency lock file
├── .github/
│   └── workflows/
│       └── ci.yml        # GitHub Actions CI
├── README.md             # Main documentation
└── .gitignore
```

## Coding Standards

- Follow Rust standard formatting (`cargo fmt`)
- Pass clippy checks (`cargo clippy -- -D warnings`)
- Write unit tests for new functionality (`cargo test`)
- Document public functions with doc comments

## Building

```bash
# Build for release
cargo build --release

# Build with specific target
cargo build --release --target x86_64-unknown-linux-gnu
```

## Testing

```bash
# Run all tests
cargo test --verbose

# Run doc tests
cargo test --doc

# Run with output
cargo test -- --nocapture
```

## Submitting Changes

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Ensure code passes formatting and clippy: `cargo fmt && cargo clippy`
5. Run tests: `cargo test`
6. Commit with a clear message
7. Push to your fork and submit a PR

## Code Review Process

- PRs require at least 1 approval
- All CI checks must pass (build, test, fmt, clippy)
- Address review feedback promptly
- Squash commits before merging

## Platform-Specific Notes

### Linux
- Requires `libasound2-dev` for audio libraries
- Requires X11 or Wayland for GUI

### macOS
- Requires Cocoa frameworks (included with Xcode)
- Uses the system clipboard API

### Windows
- Uses the Win32 clipboard API
- No additional dependencies required

## Reporting Issues

- Use GitHub Issues for bugs and feature requests
- Include your OS, Rust version (`rustc --version`), and error details
- For crashes, include a backtrace (`RUST_BACKTRACE=1 cargo run`)
