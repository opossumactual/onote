# ghostnote Design

A hardened fork of onote focused on protecting voice notes against device seizure and forensic analysis.

## Overview

### Core Security Properties

- All notes encrypted at rest with AES-256-GCM
- Master password required on launch, auto-lock after inactivity
- Recovery key generated at setup for password loss scenarios
- Audio recordings transcribed then immediately deleted (no audio persistence)
- Deleted notes are just ciphertext without the key reference - forensically useless

### What It's NOT

- Not a network anonymity tool (no Tor, no sync)
- Not protection against screen capture or keyloggers (malware on device = game over)
- Not a hidden/deniable volume (the app and encrypted files are visible, just unreadable)

### Threat Model

| Threat | Protected? |
|--------|------------|
| Device stolen while locked | Yes |
| Device seized, analyzed forensically | Yes |
| Shoulder surfing while unlocked | No |
| Malware/keylogger on device | No |
| Rubber hose cryptanalysis | No (no duress features) |

## Encryption Architecture

### Scheme

- **Algorithm:** AES-256-GCM (authenticated encryption - tamper-evident)
- **Key derivation:** Argon2id (memory-hard, resists GPU/ASIC attacks)
- **Per-note keys:** Each note gets a unique Data Encryption Key (DEK), wrapped by a master Key Encryption Key (KEK)

### Why Per-Note Keys?

If we used one key for everything, decrypting any note means having the master key in memory. With per-note keys:

- KEK derived from password, used only to unwrap DEKs
- DEKs stored encrypted alongside each note
- Deleting a note = deleting its wrapped DEK = that note is gone forever, even if ciphertext recovered

### File Structure

```
~/Documents/ghostnote/
├── .vault/
│   ├── salt              # Random salt for Argon2
│   ├── verify            # Encrypted test blob to verify password
│   └── recovery.key      # Recovery key (encrypted, shown once at setup)
├── inbox/
│   ├── note-abc123.enc   # Encrypted note content
│   └── note-abc123.key   # Wrapped DEK for this note
└── work/
    └── ...
```

### What's NOT Encrypted

- Folder names (visible in filesystem)
- Note file count and sizes (metadata leakage, but acceptable tradeoff for simplicity)

## Key Management

### First Launch (Setup Flow)

1. User creates master password
2. App generates:
   - Random 256-bit salt (stored in `.vault/salt`)
   - KEK derived via Argon2id(password, salt, cost params)
   - Random recovery key (displayed as `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`)
   - Recovery key encrypted with KEK, stored in `.vault/recovery.key`
3. User shown recovery key once with prompt: "Save this in your password manager. You won't see it again."
4. Verification blob encrypted and stored (to check password correctness on future unlocks)

### Normal Unlock

1. User enters password
2. Derive KEK from password + stored salt
3. Attempt to decrypt verification blob
4. If success: app unlocked, KEK held in memory
5. If fail: "Wrong password" error

### Recovery Flow

1. User clicks "Forgot password"
2. Enters recovery key
3. Recovery key decrypts stored recovery blob, reveals original KEK
4. User sets new password
5. KEK re-wrapped with new password-derived key
6. New recovery key generated (old one invalidated)

### Password Change

Same as recovery - re-derive KEK, re-wrap everything.

## Auto-Lock Behavior

### When Does It Lock?

- After N minutes of inactivity (default: 5 minutes)
- Configurable in settings: 1, 5, 10, 15, 30 minutes
- On system sleep/suspend

### What Happens on Lock?

1. Auto-save triggers (encrypts current editor content to disk)
2. KEK cleared from memory
3. Any decrypted note content in editor cleared
4. Decrypted DEKs in memory cleared
5. UI switches to lock screen (password prompt)

No data loss - work is always saved before locking.

### What Counts as Activity?

- Keystrokes in editor
- Mouse clicks
- Recording audio
- NOT: just having the window open/focused

### Lock Screen UI

- Simple password field, centered
- "Forgot password?" link for recovery flow
- No indication of what's inside (no note count, no folder names)
- App name/logo only

### Edge Case: Recording in Progress

- If auto-lock triggers during recording, finish transcription first
- Then lock immediately after text is encrypted and saved
- Don't lose the user's voice note mid-sentence

## Audio Handling

### Recording Flow

1. User presses record button (or Ctrl+R)
2. Audio captured to memory buffer (not written to disk)
3. User stops recording
4. Audio sent directly to Whisper for transcription
5. Transcribed text inserted at cursor position (or new note)
6. Audio buffer cleared from memory
7. No WAV file ever written to disk

### Why Memory-Only?

- Current onote writes WAV to `~/Documents/opnotes/.audio/`
- Those files could be forensically recovered even after deletion
- ghostnote never writes audio to disk - nothing to recover

### Tradeoffs

- Can't replay the original recording later
- If transcription is bad, you can't re-listen and fix it
- Larger recordings need more RAM temporarily (1 min ≈ 2 MB)

### Implementation Change

- Remove `save_recording()` disk write from `audio.rs`
- Pass audio buffer directly to Whisper via channel/shared memory
- Clear buffer after transcription completes

## UI Changes

### Lock Screen (New)

- Full-screen overlay when locked
- ghostnote logo/name (subtle, minimal)
- Password input field, centered
- "Unlock" button
- "Forgot password?" link (small, below)
- No hints about content inside

### Setup Wizard (New, First Launch Only)

- Step 1: "Create a master password" (with strength indicator)
- Step 2: "Save your recovery key" (displayed prominently, copy button, checkbox: "I've saved this")
- Step 3: "You're ready" → enters app

### Settings Additions

- Auto-lock timeout dropdown (1, 5, 10, 15, 30 min)
- "Lock now" button
- "Change password" option
- "View recovery key" (requires current password, regenerates new one)

### Visual Identity

- New app name: "ghostnote" in toolbar
- New icon (ghost-themed)
- Muted, covert color palette (dark grays, desaturated accents)

### Removed from onote

- Audio playback UI (no recordings to play)
- `.audio` folder references

## Migration

### Fresh Install

Standard setup wizard (password, recovery key), empty vault, ready to use.

### Existing onote User

Manual migration:

1. Export notes from onote (they're just markdown files)
2. First launch of ghostnote → setup wizard
3. Import option: "Import existing notes folder"
4. ghostnote reads markdown files, encrypts each one, stores in vault
5. User manually deletes old unencrypted onote folder

### Why Not Auto-Migrate?

- Security decision should be deliberate
- User needs to securely delete old unencrypted files themselves
- Avoids accidentally leaving unencrypted copies around

### Data Format

- Encrypted notes are NOT compatible with onote
- No "export unencrypted" feature (intentional)
- If user wants out, they copy/paste manually while unlocked

## Technical Implementation

### Rust Crates

- `aes-gcm` - AES-256-GCM encryption/decryption
- `argon2` - Password key derivation
- `rand` - Cryptographically secure random generation
- `zeroize` - Secure memory clearing (wipe keys from RAM)

### New Tauri Commands

```rust
unlock_vault(password) -> Result<(), Error>
lock_vault() -> ()
setup_vault(password) -> RecoveryKey
recover_vault(recovery_key, new_password) -> NewRecoveryKey
change_password(old, new) -> NewRecoveryKey
get_lock_status() -> { locked: bool, timeout_remaining: u32 }
```

### Modified Commands

- `read_note(path)` - now decrypts before returning content
- `save_note(path, content)` - now encrypts before writing
- `delete_note(path)` - deletes both `.enc` and `.key` files
- `start_recording()` - no longer writes to disk
- `stop_recording()` - returns audio buffer, clears memory

### State Management

- KEK stored in Tauri app state (behind Mutex)
- `zeroize` trait implemented so KEK is wiped on drop
- Auto-lock timer runs in background thread

### Frontend Changes

- New `vault.svelte.ts` store for lock state
- Lock screen component intercepts all UI when locked
- Settings updated for timeout config

## Not In Scope (For Now)

- Panic/duress features
- Hidden volumes
- Network sync
- Encrypted folder names

## Components to Build

1. Lock screen UI + setup wizard
2. Vault management (Rust encryption layer)
3. Auto-lock timer system
4. Memory-only audio pipeline
5. Settings for timeout config
6. New theme palette with muted colors
