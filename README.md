# BentoSSH

> **A modern, privacy-first SSH client for developers who care about craft.**
> Inspired by the workflow philosophy of Termius, rebuilt from the ground up in Rust + Tauri + SvelteKit.

## Features

- **Bento Grid Dashboard** — Spatial card-based layout showing active terminals, server health, quick actions, and pinned hosts
- **Native Performance** — Cold start <500ms on Apple Silicon. No Electron overhead
- **Privacy-First** — No cloud sync. No telemetry. Credentials stored in OS-native keychain (Keychain/libsecret/Windows Credential Manager)
- **Integrated Terminal** — Xterm.js + PTY streaming, full theme integration, resize support
- **Multi-Session Tiling** — Run multiple SSH sessions in tiled bento boxes within a single window

## Tech Stack

**Backend:** Rust + Tauri v2 + ssh2 + tokio + keyring-rs + rusqlite  
**Frontend:** SvelteKit 5 + Tailwind CSS + Xterm.js + lucide-svelte  
**Build:** Cross-platform via GitHub Actions (macOS, Linux, Windows)

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (v20+)
- [Tauri CLI](https://tauri.app/v2/start/prerequisites/)

### Development

```bash
# Clone repo
git clone https://github.com/pallab-js/BentoSSH.git
cd bentossh

# Install dependencies
npm install

# Run in dev mode
npm run tauri:dev
```

### Testing

```bash
# Run backend integration tests
npm test
```

### Build

```bash
# Build for production
npm run tauri:build

# Output: src-tauri/target/release/bundle/
```

## Project Structure

```
bentossh/
├── src-tauri/          # Rust backend
│   ├── src/
│   │   ├── lib.rs     # Tauri commands (ssh_connect, etc.)
│   │   └── tests.rs    # Backend integration tests
│   └── Cargo.toml
├── src/                # SvelteKit frontend
│   ├── routes/
│   │   └── +page.svelte  # Main Bento Grid dashboard
│   └── lib/
│       └── components/
│           └── Terminal.svelte  # Xterm.js wrapper
├── docs/               # Project documentation (Blueprint, Design)
├── src-tauri/target/release/bundle/  # Built apps
└── .github/workflows/  # CI/CD
```

## Architecture

```
┌─────────────────────────────────────────┐
│             SvelteKit Frontend           │
│  Bento Grid · Xterm.js · Tailwind CSS   │
└────────────────────┬────────────────────┘
                     │  Tauri IPC (Commands / Events)
┌────────────────────▼────────────────────┐
│              Rust Backend                │
│  SSH Engine · Keychain · SQLite · PTY   │
└─────────────────────────────────────────┘
```

## Security

- **Zero-Trust Storage** — Host metadata in SQLite. Passwords/keys in OS keychain only.
- **Isolation Pattern** — Frontend can only call declared `#[tauri::command]` functions.
- **Path Traversal Protection** — SSH key paths are strictly validated and restricted to `~/.ssh` or the app's internal directory.
- **SSRF Mitigation** — Outbound SSH connections are restricted for sensitive internal ports to prevent network scanning.
- **Restricted Permissions** — Application data directory (`~/.bentossh`) and database file are created with strict `0700` and `0600` permissions on Unix systems.
- **No Telemetry** — App runs fully local. No data leaves your machine.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

GPL-3.0 — Open Source. Contributions welcome.

---

**Status:** Production Ready (v1.0.0)  
**Current Version:** 1.0.0  
**Platforms:** macOS (Apple Silicon & Intel), Linux, Windows
