# Clipboard GUI

Cross-platform clipboard history manager built with Rust and egui.

## Features

- Monitor clipboard changes in real-time
- Store up to 100 history items with persistence
- Search through clipboard history
- One-click copy from history
- Delete individual items or clear all
- Automatic storage to local file

## Dependencies

- Rust 1.75+
- Cargo

## Build

```bash
cargo build --release
```

Binary will be at `target/release/clipboard-gui`.

## Usage

Run the binary and the GUI will open automatically.

## Releases

Pre-built binaries for Linux and Windows are attached to each
[GitHub release](https://github.com/CREATSAIF/clipboard-gui/releases).
Download the artifact for your platform, then run it directly — no
install step required.

| Platform | File |
| --- | --- |
| Linux x86_64 | `clipboard-gui-linux-x86_64` |
| Windows x86_64 | `clipboard-gui-windows-x86_64.exe` |

Each release also includes a `.sha256` checksum file.

## License

MIT
