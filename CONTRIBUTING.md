# Contributing to BentoSSH

Thank you for your interest in contributing! This guide will help you get started.

## Development Setup

1. **Fork & Clone**
   ```bash
   git clone https://github.com/yourusername/bentossh.git
   cd bentossh
   ```

2. **Install Dependencies**
   ```bash
   npm install
   ```

3. **Run in Development Mode**
   ```bash
   npm run tauri:dev
   ```

## Project Guidelines

### Code Style

- **Rust:** Follow `rustfmt` formatting. Run `cargo fmt` before committing.
- **Svelte/TypeScript:** Use Tailwind utility classes. No external CSS files.
- **Commit Messages:** Use [Conventional Commits](https://www.conventionalcommits.org/). Example: `feat: add SFTP browser tile`.

### Architecture Rules

- **Tauri Commands:** Must return `Result<T, String>`. Use `thiserror` for custom error types.
- **State Management:** Use Svelte 5 runes (`$state`, `$derived`, `$effect`). No Svelte 4 stores.
- **Credential Security:** Never store plaintext credentials anywhere except OS keychain via `keyring-rs`.
- **Async SSH:** Never block async runtime. Use `tokio::spawn_blocking` for synchronous SSH operations.

### Pull Request Process

1. Create a feature branch (`git checkout -b feat/amazing-feature`)
2. Make your changes
3. Run linting and tests:
   ```bash
   # Rust
   cd src-tauri && cargo clippy
   
   # Frontend
   npm run check
   ```
4. Commit with a conventional message
5. Push to your fork and open a Pull Request

### Adding New Features

- Follow the **Vertical Slice** strategy: build one complete feature at a time.
- Update the blueprint document (`docs/blueprint.md`) if architecture changes.
- Add appropriate Tauri commands and frontend components.

### Reporting Issues

When reporting bugs, please include:
- OS and version (macOS, Linux, Windows)
- Steps to reproduce
- Expected vs actual behavior
- Any error messages from the console

## Testing

- **Manual Testing:** Test SSH connections with a real server. Verify key-based and password auth.
- **UI Testing:** Check Bento Grid layout at different window sizes.
- **Security Testing:** Ensure no credentials appear in SQLite or logs.

---

**Questions?** Open an issue or reach out to the maintainers.
