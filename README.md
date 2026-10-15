<div align="center">

# BentoSSH

A modern, privacy-first SSH client built with Rust, Tauri, and SvelteKit.

[![CI](https://github.com/pallab-js/BentoSSH/actions/workflows/ci.yml/badge.svg)](https://github.com/pallab-js/BentoSSH/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)

</div>

## Overview

BentoSSH is a lightweight desktop SSH client. It stores host metadata locally in
SQLite and keeps credentials in your operating system's native keychain. There is
no cloud sync and no telemetry — everything runs on your machine.

> **Status:** Early development (`0.1.0`). Core connect, terminal, and host
> management flows work. Expect rough edges.

## Features

- **Bento grid dashboard** — card-based layout for hosts, sessions, and quick actions.
- **Integrated terminal** — Xterm.js front end streaming a real PTY over SSH, with resize support.
- **Multiple sessions** — open several SSH sessions as tabs in one window.
- **Native keychain storage** — passwords and keys live in Keychain (macOS),
  libsecret (Linux), or Credential Manager (Windows), never in plain text.
- **Local-only** — host metadata in a `0600` SQLite file under `~/.bentossh`; no network calls beyond your SSH connections.

## Tech stack

| Layer    | Technologies                                              |
| -------- | --------------------------------------------------------- |
| Backend  | Rust, Tauri v2, `ssh2`, `tokio`, `keyring`, `rusqlite`    |
| Frontend | SvelteKit 5, Tailwind CSS, Xterm.js, lucide-svelte        |
| CI       | GitHub Actions (lint, test, cross-platform build)         |

## Getting started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 20+
- Platform build dependencies — see the [Tauri prerequisites](https://tauri.app/start/prerequisites/)

### Development

```bash
git clone https://github.com/pallab-js/BentoSSH.git
cd BentoSSH
npm install
npm run tauri:dev
```

### Build

```bash
npm run tauri:build
# Bundles are written to src-tauri/target/release/bundle/
```

## Testing & quality

```bash
npm test                       # Rust backend tests (cargo test)
npm run check                  # Svelte / TypeScript type-check
cd src-tauri && cargo clippy   # Rust linting
cd src-tauri && cargo fmt      # Rust formatting
```

CI runs formatting, Clippy (`-D warnings`), backend tests, a frontend build, and
`svelte-check` on every push and pull request to `main`.

## Project structure

```
BentoSSH/
├── src/                       # SvelteKit frontend
│   ├── routes/                # Pages and layout
│   └── lib/
│       ├── components/        # Terminal, command palette, settings, health
│       └── *.svelte.ts        # Session / UI / settings state
├── src-tauri/                 # Rust backend
│   └── src/
│       ├── lib.rs             # Tauri commands (ssh_connect, host CRUD, …)
│       ├── tests.rs           # Backend tests
│       └── main.rs
├── docs/                      # Blueprint and design notes
└── .github/workflows/         # CI and release pipelines
```

## Security

- **Credential isolation** — secrets are stored only in the OS keychain; the
  SQLite database holds non-secret host metadata.
- **Command isolation** — the frontend can only call declared
  `#[tauri::command]` functions over Tauri IPC.
- **Key path validation** — SSH key paths are canonicalized and restricted to
  `~/.ssh` or `~/.bentossh`.
- **Port restrictions** — outbound connections to a set of common non-SSH ports
  are blocked to reduce accidental network scanning.
- **Strict file permissions** — on Unix, `~/.bentossh` is created `0700` and the
  database file `0600`.

Found a vulnerability? Please open a private security advisory rather than a public issue.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

[GPL-3.0](LICENSE)
