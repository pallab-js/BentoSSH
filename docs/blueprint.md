# BentoSSH — Project Blueprint

> **A modern, privacy-first SSH client for developers who care about craft.**
> Inspired by the workflow philosophy of Termius, rebuilt from the ground up in Rust + Tauri + SvelteKit.

---

**Status:** MVP Blueprint — Solo Developer Build  
**Stack:** Rust · Tauri v2 · SvelteKit · Tailwind CSS  
**Target Platforms:** macOS (M1 Optimized) · Linux · Windows  
**License:** GPL-3.0 (Open Source)

---

## 1. Executive Summary

BentoSSH is a high-performance, privacy-first desktop SSH client that reimagines how developers interact with remote servers. Taking direct inspiration from Termius — the gold standard for multi-platform SSH UX — BentoSSH differentiates itself by:

- **Running fully local.** No accounts, no cloud sync by default, no telemetry. Your keys and credentials never leave your machine.
- **A Bento Grid dashboard.** Instead of a flat list of connections (the dominant legacy pattern), BentoSSH surfaces your workflow in a spatial, card-based grid — active terminals, server health, quick actions, and pinned hosts, all visible at once.
- **Native performance via Rust + Tauri.** Cold start under 500ms on Apple Silicon. No Electron overhead.

The aesthetic is deliberate: deep dark surfaces, glassmorphism accents, and subtle motion design that makes the app feel *alive* without being distracting.

---

## 2. Product Requirements Document (PRD)

### 2.1 Target Users

| Persona | Primary Need |
|---|---|
| **Solo Developer** | Fast access to a handful of daily servers with zero friction |
| **Privacy-Conscious User** | Guaranteed local-only credential storage; no vendor lock-in |
| **Power User / Terminal Native** | Clean multi-session tiling, keyboard-first navigation |

### 2.2 MVP Feature Set

#### Core Features

1. **Bento Dashboard**
   A 12-column responsive grid layout. Tiles include: active terminal(s), server health metrics, quick-action buttons, and a pinned hosts bar. Users can rearrange tiles per session.

2. **SSH Engine**
   Supports key-based authentication (RSA, Ed25519) and password authentication. Built on `russh` or `ssh2-rs` with a `tokio` async runtime.

3. **Secure Credential Vault**
   All sensitive credentials (passwords, private keys) are stored exclusively in the OS-native keychain via `keyring-rs`. No plaintext on disk — ever.

4. **Integrated Terminal Emulator**
   Xterm.js embedded within Svelte, connected to a Rust-managed PTY. Supports resize, scrollback, and full theme integration.

5. **Multi-Session Tiling**
   Multiple SSH sessions can run simultaneously in tiled bento boxes within a single window. Each tile is independently scrollable and interactive.

### 2.3 Success Metrics

| Metric | Target |
|---|---|
| Cold start time | < 500ms on Apple M1 |
| Connect to pinned server | ≤ 2 clicks / keystrokes |
| Plaintext credentials on disk | Zero |
| Bundle size | < 15MB |

---

## 3. Technical Requirements Document (TRD)

### 3.1 Architecture Overview

BentoSSH follows a clean separation between a **Rust backend** (SSH logic, security, system calls) and a **SvelteKit frontend** (UI, layout, terminal rendering), bridged by **Tauri v2** IPC commands.

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

### 3.2 Dependency Map

#### Backend (Rust)

| Crate | Purpose |
|---|---|
| `russh` or `ssh2-rs` | SSH protocol implementation |
| `tokio` | Async runtime |
| `keyring-rs` | OS-native keychain access (macOS Keychain, libsecret, Windows Credential Manager) |
| `rusqlite` | Local SQLite for host metadata and session history |
| `thiserror` | Structured error enums for clean Tauri command returns |
| `portable-pty` | Cross-platform PTY for terminal I/O |

#### Frontend (SvelteKit)

| Package | Purpose |
|---|---|
| `xterm.js` + `xterm-addon-fit` | Terminal emulator surface |
| `tailwindcss` | Utility-first layout (Bento Grid) |
| `lucide-svelte` | Minimal, consistent iconography |
| Svelte 5 Runes (`$state`, `$derived`) | Reactive state management |

### 3.3 Security Design

**Zero-Trust Storage Model:**
- Host metadata (alias, hostname, port, username) lives in a local SQLite database.
- Passwords and private keys are **never** written to SQLite. They are stored in and retrieved from the OS keychain exclusively.
- The Tauri **Isolation Pattern** is enforced: the frontend cannot call Rust internals arbitrarily — only declared, typed `#[tauri::command]` functions are exposed.

**Authentication Flow:**
```
User initiates connect
  → Frontend sends host_id via Tauri command
  → Rust fetches credentials from OS Keychain by host_id
  → SSH handshake occurs entirely in Rust
  → PTY stream piped back to Xterm.js via Tauri event
```

### 3.4 Error Handling Contract

All Rust commands must return `Result<T, String>` (Tauri-compatible) using `thiserror`-defined error enums. The frontend handles errors uniformly via non-blocking toast notifications — no native `alert()` dialogs.

```rust
// Rust pattern
#[derive(thiserror::Error, Debug)]
pub enum SshError {
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("Connection refused: {host}:{port}")]
    ConnectionRefused { host: String, port: u16 },
}
```

```typescript
// TypeScript pattern
type Result<T, E = string> = { ok: true; value: T } | { ok: false; error: E };
```

---

## 4. UI/UX Design

### 4.1 Visual Identity (Supabase-Inspired)

BentoSSH adopts a dark-mode-native aesthetic that channels the feel of a premium developer platform. It moves away from generic shadows in favor of a sophisticated border hierarchy and translucent layering.

| Token | Value | Usage |
|---|---|---|
| `--base` | `#171717` | Primary canvas background |
| `--surface` | `#0f0f0f` | Button backgrounds, deepest surfaces |
| `--border-subtle` | `#242424` | Horizontal rules, dividers |
| `--border-std` | `#2e2e2e` | Card borders, default edges |
| `--border-heavy` | `#363636` | Button borders, prominent dividers |
| `--accent-green` | `#3ecf8e` | Brand identity, active indicators |
| `--accent-link` | `#00c573` | Interactive elements, links |
| `--text-primary` | `#fafafa` | Primary headings and body |
| `--text-muted` | `#898989` | Labels, secondary metadata |

**Visual language:**
- **Border-Defined Depth:** No box-shadows. Elevation is communicated through increasing border brightness (Subtle → Standard → Prominent).
- **Emerald Signal:** Green is used selectively as an identity marker (logo, active session glow, primary links).
- **Pill Philosophy:** Primary CTAs use a `9999px` radius (pills), while secondary cards use `8px–16px`.
- **Translucency:** Uses HSL with alpha channels for overlays (e.g., `rgba(41, 41, 41, 0.84)` for glass effects).

### 4.2 Typography Rules

| Role | Font | Size | Weight | Line Height |
|------|------|------|--------|-------------|
| **Hero/Display** | Circular | 72px | 400 | 1.00 (Tight) |
| **Section Title** | Circular | 24px | 400 | 1.33 |
| **Technical Label** | Source Code Pro | 12px | 400 | 1.33 (Uppercase) |
| **Body/UI** | Circular | 14px | 400/500 | 1.43 |

- **Weight restraint:** Primarily weight 400. Weight 500 is reserved for buttons and navigation.
- **Monospace Ritual:** Uppercase technical labels use `Source Code Pro` with `1.2px` letter-spacing.

### 4.3 Bento Grid Layout (12-Column)

BentoSSH uses a 12-column responsive grid to surface the developer's workflow spatially.

| Tile | Columns × Rows | Content |
|---|---|---|
| **Terminal Instance** | 8 × 4 | Multiple independent terminal tiles |
| **Server Health** | 4 × 2 | CPU/RAM metrics via the active session |
| **Quick Actions** | 4 × 2 | One-click commands (restart, logs) |
| **Pinned Hosts** | 12 × 2 | Horizontally scrollable configuration cards |

### 4.4 Interaction States

| State | Visual Cue |
|---|---|
| **Connecting** | 1.5s pulse on `--border-std` towards `--accent-green` |
| **Connected** | Steady green border highlight (`rgba(62, 207, 142, 0.3)`) |
| **Active** | `ring-2 ring-accent-green` on the selected tile |
| **Error** | Red border pulse (Tomato/Crimson alpha) |


---

## 5. Development Workflow & AI-Assisted Coding

### 5.1 Vertical Slice Strategy

Build and validate one complete feature slice at a time — never scaffold the entire app at once.

| Slice | Goal |
|---|---|
| **Slice 1** | Tauri v2 + SvelteKit scaffold. Bento Grid layout with static placeholder tiles. Theme applied. |
| **Slice 2** | Rust `ssh_connect` command. Takes host/user/key path, returns `Ok` or typed error. Tested via Tauri dev tools. |
| **Slice 3** | Xterm.js component wired to Rust PTY. Input forwarding and terminal resize working. |
| **Slice 4** | Keychain integration. Credential save/fetch via `keyring-rs`. SQLite host metadata schema. |
| **Slice 5** | Multi-tab support. Session state managed in Svelte stores. Tile switching. |

### 5.2 AI Prompt Context Block

When starting a coding session with an AI assistant, paste this context block first:

```
Project: BentoSSH — a Tauri v2 desktop SSH client
Stack: Rust backend, SvelteKit frontend, Tailwind CSS, Xterm.js
Rules:
- All Rust Tauri commands must return Result<T, String>
- Use thiserror for all custom error types
- Use Svelte 5 runes ($state, $derived, $effect) — not Svelte 4 stores
- No external CSS files; Tailwind utility classes only
- No plaintext credential handling anywhere in frontend or Rust outside keyring-rs
- Prefer russh over ssh2-rs for async compatibility with tokio
```

### 5.3 Common Pitfalls to Avoid

| Risk | Mitigation |
|---|---|
| AI hallucinating deprecated Tauri v1 APIs | Always specify "Tauri v2" explicitly; validate against Tauri v2 docs |
| SSH credentials leaked into SQLite | Store only `keychain_id` reference in SQLite; never the secret itself |
| Xterm.js resize not working | Always use `xterm-addon-fit` and hook into Svelte's `onMount` + `ResizeObserver` |
| Blocking the Rust async runtime | All SSH operations must be spawned with `tokio::spawn`; never call `.block_on()` from a command |

---

## 6. Implementation Roadmap

### Phase 1 — Shell (Day 1)
- [ ] Scaffold Tauri v2 project with SvelteKit and Tailwind
- [ ] Implement 12-column Bento Grid with static placeholder tiles
- [ ] Apply full dark theme (CSS variables)
- [ ] Add system/dark theme toggle

### Phase 2 — Rust Core (Days 2–3)
- [ ] Implement `ssh_connect` Tauri command (key-based auth)
- [ ] Define SQLite schema for host metadata (`hosts` table)
- [ ] Integrate `keyring-rs` for credential storage and retrieval
- [ ] Add `thiserror` error types for all SSH failure modes

### Phase 3 — Terminal Integration (Days 4–5)
- [ ] Embed Xterm.js in a Svelte component
- [ ] Implement PTY in Rust using `portable-pty`
- [ ] Wire Tauri events for stdin/stdout streaming
- [ ] Handle terminal resize via `xterm-addon-fit` + Tauri event

### Phase 4 — Polish & Release (Days 6–7)
- [ ] Add tile animations (Svelte transitions + CSS keyframes)
- [ ] Implement connection state indicators (glow / pulse animations)
- [ ] Configure GitHub Actions for cross-platform builds (`.app`, `.dmg`, `.deb`, `.exe`)
- [ ] Write `README.md`, `CONTRIBUTING.md`, and `LICENSE`
- [ ] Publish initial release to GitHub

---

## 7. Open Questions & Future Scope

| Item | Notes |
|---|---|
| **SFTP / File Browser** | Post-MVP; would require a dedicated Bento tile |
| **SSH Tunneling / Port Forwarding** | High-value feature; add to Phase 2 backlog |
| **Cloud Sync (Opt-in)** | E2E-encrypted; only if there's user demand |
| **Mobile (iOS/Android)** | Out of scope for desktop MVP |
| **Plugins / Scripting** | Consider a Lua or WASM plugin layer in v2 |

---

*BentoSSH is open source under GPL-3.0. Contributions welcome.*
