# BentoSSH Guide

## Build Commands
- `npm run dev`: Start SvelteKit dev server (standalone)
- `npm run tauri:dev`: Start Tauri dev environment (Frontend + Backend)
- `npm run build`: Build SvelteKit frontend
- `npm run tauri:build`: Build production application
- `npm run check`: Run svelte-check for frontend types
- `cd src-tauri && cargo clippy`: Lint Rust backend
- `cd src-tauri && cargo fmt`: Format Rust backend

## Code Guidelines
- **Frontend**: Svelte 5 with Runes (`$state`, `$derived`, `$effect`). Tailwind CSS for styling.
- **Backend**: Rust with Tauri v2. SSH logic using `ssh2` or `russh`.
- **Security**: Never store plaintext credentials. Use OS keychain via `keyring-rs`.
- **Formatting**: `cargo fmt` for Rust, Prettier (if configured) for JS/Svelte.
- **Commands**: Tauri commands must return `Result<T, String>`.

## Documentation
- `docs/blueprint.md`: Core project architecture and requirements.
- `docs/design.md`: Visual and UI/UX design specifications.
