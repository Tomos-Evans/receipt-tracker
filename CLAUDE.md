# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Development server (localhost:8080, hot reload)
trunk serve -a 0.0.0.0 --public-url /receipt-tracker/ --open=false

# Production build
trunk build --release --public-url /receipt-tracker/

# Fast compile check (no codegen)
cargo check --target wasm32-unknown-unknown

# Format, then lint (run both after any code change)
cargo fmt
cargo clippy --target wasm32-unknown-unknown

# To run tests using headless firefox
wasm-pack test --headless --firefox
```

`cargo clippy` must run after every edit — fix all warnings before finishing.

## Architecture Overview

Mobile-first PWA built with Yew (Rust/WASM). All data in IndexedDB via rexie; no backend, no cloud sync. Deployed as static files.

