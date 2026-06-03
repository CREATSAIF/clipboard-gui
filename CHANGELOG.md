# Changelog

All notable changes to clipboard-gui are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-03

### Added
- Cross-platform clipboard history manager built with Rust + egui 0.29
- eframe desktop frontend (one binary, no extra runtime)
- Real-time clipboard monitoring with 500 ms polling
- Persistent history (up to 100 items) stored in
  `dirs::data_local_dir()/clipboard-gui/clipboard_history.json`
- Searchable history list with character-count and preview metadata
- Per-item copy back to the system clipboard
- Per-item delete and clear-all
- GitHub Actions CI: build, test, fmt --check, clippy (-D warnings)
- GitHub Actions release workflow: cross-platform build (Linux + Windows)
  on `v*` tag pushes with SHA-256 checksums and auto-generated release notes
