# CLAUDE.md - ghostnote

A hardened, encrypted voice notes app with local Whisper transcription. Notes are encrypted at rest with AES-256-GCM and audio never touches disk.

## Tech Stack

- **Frontend**: Svelte 5 (runes: `$state`, `$derived`, `$effect`, `$props`)
- **Backend**: Tauri v2 (Rust)
- **Audio**: cpal for cross-platform audio capture
- **Transcription**: whisper-rs for local speech-to-text
- **Encryption**: AES-256-GCM with Argon2id key derivation
- **Storage**: Encrypted notes in `~/Documents/ghostnote/`

## Security Architecture

### Encryption Overview
- **Master Password** → Argon2id (64MB, 3 iterations, 4 lanes) → **KEK** (Key Encryption Key)
- **Per-note DEK** (Data Encryption Key) → wrapped by KEK → stored in `.key` file
- **Note content** → AES-256-GCM encrypted → stored in `.enc` file
- **Recovery key**: 24 base64 chars (XXXX-XXXX-XXXX-XXXX-XXXX-XXXX)

### Memory-Only Audio
- Audio samples stored only in ring buffer during recording
- Transcription receives samples directly, never written to disk
- Samples cleared from memory after transcription (zeroize)

### Auto-Lock
- Configurable timeout (1, 5, 10, 15, 30 minutes)
- Activity tracking resets timer (keystrokes, clicks)
- KEK cleared from memory on lock
- Backend emits `vault-locked` event to frontend

## Project Structure

```
src/
├── App.svelte              # Main app, vault status routing, activity tracking
├── lib/
│   ├── components/
│   │   ├── Toolbar.svelte        # Top bar with window controls
│   │   ├── Sidebar.svelte        # Folder tree, search
│   │   ├── NoteList.svelte       # Note list
│   │   ├── Editor.svelte         # Markdown editor
│   │   ├── StatusBar.svelte      # Word count, save status
│   │   ├── RecordButton.svelte   # Recording button
│   │   ├── Settings.svelte       # Audio, model, theme, vault settings
│   │   ├── LockScreen.svelte     # Password unlock screen
│   │   ├── SetupWizard.svelte    # First-run vault setup (3-step)
│   │   └── KeyboardShortcuts.svelte
│   ├── stores/
│   │   ├── notes.svelte.ts       # Notes & folders state
│   │   ├── editor.svelte.ts      # Editor content, save
│   │   ├── recording.svelte.ts   # Recording state (memory-only audio)
│   │   ├── theme.svelte.ts       # Theme state
│   │   ├── vault.svelte.ts       # Vault lock/unlock state
│   │   └── ui.svelte.ts          # UI state (panels, settings)
│   ├── themes.ts                 # Theme definitions (7 themes, covert default)
│   └── utils/
│       └── tauri-commands.ts     # Typed Tauri command wrappers

src-tauri/
├── src/
│   ├── lib.rs                    # App state, Tauri setup, auto-lock timer
│   └── commands/
│       ├── mod.rs
│       ├── notes.rs              # Encrypted CRUD for notes
│       ├── settings.rs           # App settings
│       ├── audio.rs              # Memory-only recording
│       ├── whisper.rs            # Transcription from buffer
│       └── vault.rs              # Encryption, KEK, vault state
├── capabilities/
│   └── default.json
├── Cargo.toml
└── tauri.conf.json
```

## Key Commands

```bash
# Development
npm run tauri dev        # Start dev server with hot reload

# Build
npm run tauri build      # Build production release

# Rust checks
cd src-tauri && cargo check
```

## Vault Commands (Rust)

| Command | Description |
|---------|-------------|
| `setup_vault` | Create vault with password, returns recovery key |
| `is_vault_setup` | Check if vault exists |
| `unlock_vault` | Unlock with password |
| `lock_vault` | Lock vault (clears KEK) |
| `get_vault_status` | Returns { initialized, locked, timeout_remaining } |
| `vault_activity` | Reset auto-lock timer |
| `set_lock_timeout` | Set timeout in seconds |
| `recover_vault` | Reset password with recovery key |
| `change_password` | Change password (requires current password) |

## Vault Files

Located in `~/Documents/ghostnote/.vault/`:
- `salt` - 32-byte random salt for Argon2id
- `verify` - Encrypted verification blob
- `recovery.key` - JSON with encrypted KEK for recovery

## Note Storage

Each encrypted note consists of:
- `<note-name>.enc` - AES-256-GCM encrypted content (nonce prepended)
- `<note-name>.key` - AES-256-GCM wrapped DEK

Unencrypted `.md`/`.txt` files are still readable for backward compatibility.

## Theme System

7 built-in themes with localStorage persistence:

| Theme | Type | Description |
|-------|------|-------------|
| Covert (default) | Dark | Muted slate blue, low contrast |
| Coral Terminal | Dark | Warm orange accent |
| Synthwave | Dark | Pink/purple accent |
| Amber Terminal | Dark | Amber/gold accent |
| Matrix | Dark | Neon green |
| Light Classic | Light | Blue accent |
| Light Warm | Light | Orange accent |

## Architecture Notes

### Vault State (vault.rs)
- Thread-safe via `Mutex<VaultStateInner>`
- KEK stored in memory only when unlocked
- `with_kek()` helper for safe KEK access
- Zeroize trait for secure memory clearing

### Audio Recording (audio.rs)
- 60-second ring buffer at 16kHz
- Returns `AudioSamples { samples: Vec<f32>, sample_rate: u32 }`
- No file I/O, no temporary files

### Whisper Transcription (whisper.rs)
- Accepts `Vec<f32>` samples directly
- Samples cleared after transcription
- Wrapped in `panic::catch_unwind` for stability

### Activity Tracking (App.svelte)
- Throttled to once per second
- Tracks keystrokes and clicks
- Calls `vault_activity` to reset auto-lock

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+B | Toggle sidebar |
| Ctrl+L | Toggle note list |
| Ctrl+N | New note |
| Ctrl+R | Toggle recording |
| Ctrl+S | Save note |
| Ctrl+/ | Show shortcuts |
| Delete | Delete selected note |
| Esc | Close dialogs |

## Data Storage

- Notes: `~/Documents/ghostnote/<folder>/<note>.enc` + `.key`
- Vault: `~/Documents/ghostnote/.vault/` (salt, verify, recovery.key)
- Models: `~/.local/share/ghostnote/models/ggml-<model>.bin`
- Config: `~/.config/ghostnote/config.json`
- Theme: `localStorage` key `opnotes-theme`

## Dependencies (Cargo.toml)

```toml
# Encryption
aes-gcm = "0.10"         # AES-256-GCM encryption
argon2 = "0.5"           # Argon2id key derivation
zeroize = "1.8"          # Secure memory clearing
rand = "0.8"             # Cryptographic RNG
base64 = "0.22"          # Recovery key encoding

# Audio/Transcription
whisper-rs = "0.14"      # Local Whisper inference
cpal = "0.15"            # Cross-platform audio
ringbuf = "0.4"          # Thread-safe audio buffer

# Other
reqwest = "0.12"         # Model downloading
tokio = "1"              # Async runtime
```

## Known Issues / TODOs

- Dynamic height issue needs fixing (from original opnotes)
- ALSA errors on Linux with PipeWire (use "pipewire" or "default" device)
- Live/streaming transcription not implemented (batch mode only)
