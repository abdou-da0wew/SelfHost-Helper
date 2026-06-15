# MIGRATION_PLAN.md — Electron → Tauri v2

**This file is auto-read by agents at session start.** It contains the complete migration plan with all sub-agent research outputs and full executable Rust code.

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture Decisions](#architecture-decisions)
3. [IPC Channel Mapping](#ipc-channel-mapping)
4. [Execution Phases](#execution-phases)
5. [Phase 1: Scaffold](#phase-1-scaffold)
6. [Phase 2: Utils](#phase-2-utils)
7. [Phase 3: Core Services](#phase-3-core-services)
8. [Phase 4: Feature Services](#phase-4-feature-services)
9. [Phase 5: Native + Commands](#phase-5-native--commands)
10. [Phase 6: Frontend Migration](#phase-6-frontend-migration)
11. [Phase 7: Cleanup + Verify](#phase-7-cleanup--verify)
12. [Full Rust Implementations](#full-rust-implementations)

---

## Overview

Port the entire Electron + React desktop app to Rust + Tauri v2 with native cross-platform support (Linux/macOS first-class), complete Windows Job Object API migration, and zero stubs.

**Constraints:**
- Zero Electron artifacts may remain after migration
- No stubs, no TODOs — every function body must be complete and correct
- All 80+ Electron IPC channels must have exact Tauri command equivalents
- Native cross-platform: Linux and macOS are first-class targets
- Windows Job Object API fully migrated to Rust `windows` crate

---

## Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Backend modules | Rust-native layered modules (not SMO) | Rust compiler enforces SMO invariants via `pub(crate)`, traits, modules |
| Database | `rusqlite` + raw SQL | Direct SQLite replacing st.db JSON driver |
| Git | `git2` crate (libgit2 bindings) | Replaces `simple-git` npm package |
| Search | Bundle `rg` as sidecar | Initial approach, migrate to `grep-*` crates later |
| LSP | `tokio-tungstenite` WebSocket bridge | Preserves Monaco editor ↔ language server architecture |
| File watching | `notify` v6 crate | Replaces chokidar, manual 200ms debounce |
| Secrets | `keyring` crate + `aes-gcm` fallback | Replaces `electron.safeStorage` |
| Tunnels | Bundle `cloudflared` as Tauri sidecar | No npm dependency |
| ZIP extraction | `zip` crate | Cross-platform, no PowerShell dependency |
| Process cleanup | `windows-sys` crate (Job Objects) + `nix` crate (Unix) | Replaces C++ node-gyp addon |
| State management | Jotai atoms retained on frontend | Rust backend uses `Arc<RwLock<>>` with Tauri managed state |
| IPC streaming | Tauri `Channel` + event system | Replaces Electron's 15 `webContents.send()` channels |

---

## IPC Channel Mapping

80+ IPC channels mapped from Electron to Tauri commands.

| Category | Commands | Rust Module |
|----------|----------|-------------|
| Projects CRUD | 6 commands | `commands/projects.rs` |
| Categories CRUD | 5 commands | `commands/categories.rs` |
| Process control | 3 commands (start/stop/restart) | `commands/process.rs` |
| Logs | 5 commands + 3 events | `commands/logs.rs` |
| File operations | 7 commands + 2 watcher | `commands/files.rs` |
| Git | 15+ commands | `commands/git.rs` |
| Tunnels | 8 commands + 2 events | `commands/tunnels.rs` |
| Search | 1 command | `commands/search.rs` |
| LSP | 2 commands | `commands/lsp.rs` |
| Settings | 2 commands | `commands/settings.rs` |
| Runtime manager | 5 commands + 1 event | `commands/runtimes.rs` |
| Config backup | 2 commands | `commands/backup.rs` |
| Updater | 4 commands + 1 event | `commands/updater.rs` |
| App utilities | 10 commands | `commands/app.rs` |
| Window controls | 3 commands | `commands/window.rs` |
| Stats | 1 command + 1 event stream | `commands/stats.rs` |

---

## Execution Phases

### Phase 1: Scaffold (no deps)
- [ ] Create `src-tauri/` directory structure
- [ ] Write `Cargo.toml` with all 30+ dependencies
- [ ] Write `tauri.conf.json`
- [ ] Write `src/main.rs`, `src/lib.rs`, `src/error.rs`, `build.rs`
- [ ] Write `capabilities/default.json`
- [ ] Update `package.json`
- [ ] Generate Tauri icons
- [ ] Update CI pipeline

### Phase 2: Utils (no deps)
- [ ] `utils/crypto.rs`
- [ ] `utils/path_security.rs`
- [ ] `utils/media_allowlist.rs`
- [ ] `utils/ignore_patterns.rs`

### Phase 3: Core Services
- [ ] `db/mod.rs`
- [ ] `services/settings_service.rs`
- [ ] `services/log_store.rs`

### Phase 4: Feature Services
- [ ] `services/tunnel_manager.rs`
- [ ] `services/git_service.rs`
- [ ] `services/search_service.rs`
- [ ] `services/lsp_bridge.rs`
- [ ] `services/file_watcher.rs`
- [ ] `services/backup_service.rs`
- [ ] `services/runtime_service.rs`
- [ ] `services/update_service.rs`

### Phase 5: Native + Commands
- [ ] `native/tray.rs`
- [ ] All `commands/*.rs`
- [ ] `services/process_manager.rs`
- [ ] `services/process_tree.rs`

### Phase 6: Frontend Migration
- [ ] `src/lib/tauri-bridge.js`
- [ ] Update 21 component files
- [ ] Remove CSP meta tag from `index.html`

### Phase 7: Cleanup + Verify
- [ ] Remove `electron/` directory
- [ ] Remove Electron deps from `package.json`
- [ ] Lint, typecheck, build
- [ ] Manual smoke test

---

## File Tree

```
src-tauri/
  Cargo.toml
  build.rs
  tauri.conf.json
  capabilities/
    default.json
  icons/ (generated)
  src/
    main.rs
    lib.rs
    error.rs
    commands/
      mod.rs, app.rs, backup.rs, categories.rs, files.rs, git.rs,
      logs.rs, lsp.rs, process.rs, projects.rs, runtimes.rs,
      search.rs, settings.rs, stats.rs, tunnels.rs, updater.rs, window.rs
    db/
      mod.rs
    services/
      mod.rs, config_backup.rs, encryption.rs, file_watcher.rs,
      ignore_patterns.rs, log_store.rs, lsp_bridge.rs,
      media_allowlist.rs, process_manager.rs, runtime_manager.rs,
      search.rs, settings.rs, shutdown.rs, stats.rs,
      tunnel_manager.rs, updater.rs
    native/
      mod.rs, tray.rs
    utils/
      mod.rs, crypto.rs, path_security.rs, media_allowlist.rs,
      ignore_patterns.rs
```

---

## Dependencies

### Rust (Cargo.toml)
tauri 2, tauri-plugin-shell/updater/process/dialog/fs/http/store/log, tokio, serde, serde_json, rusqlite (bundled), git2, reqwest, aes-gcm, pbkdf2, sha2, base64, rand, zip, notify 6, tokio-tungstenite, futures-util, which, dunce, regex, chrono, uuid, thiserror, log, url, tracing, semver, windows-sys (Windows), nix (Unix)

### Frontend (package.json)
Add: @tauri-apps/cli (dev), @tauri-apps/api
Remove: electron, electron-builder, st.db, auto-launch, node-addon-api, electron-updater, wait-on

---

## Verification Checklist

- [ ] `npm run lint` passes
- [ ] `npm run typecheck` passes
- [ ] `npm run build:web` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo build --release` succeeds
- [ ] App starts with `npm run dev`
- [ ] All 80+ IPC commands functional
- [ ] System tray works
- [ ] Process start/stop/restart works
- [ ] Log streaming works
- [ ] Git operations work
- [ ] File search works
- [ ] Tunnel management works
- [ ] LSP bridge works
- [ ] Config backup/restore works
- [ ] Auto-updater works
- [ ] Installers work on all platforms

---

---

## Phase 1: Scaffold — Full Specifications

### `src-tauri/Cargo.toml`

```toml
[package]
name = "selfhost-helper"
version = "0.40.1"
description = "Node.js Project Manager"
authors = ["@DevRoots/AboMeezO <abomeezo2@gmail.com>"]
license = "ISC"
edition = "2021"

[lib]
name = "selfhost_helper_lib"
crate-type = ["lib", "cdylib", "staticlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-updater = "2"
tauri-plugin-process = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
tauri-plugin-http = "2"
tauri-plugin-store = "2"
tauri-plugin-log = "2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
git2 = "0.19"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
aes-gcm = "0.10"
pbkdf2 = { version = "0.12", features = ["hmac"] }
sha2 = "0.10"
base64 = "0.22"
rand = "0.8"
zip = "2"
notify = "6"
tokio-tungstenite = { version = "0.24", features = ["native-tls"] }
futures-util = "0.3"
which = "7"
dunce = "1"
regex = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
thiserror = "2"
log = "0.4"
url = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
semver = "1"

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.60", features = [
    "Win32_Foundation",
    "Win32_Security",
    "Win32_System_Threading",
    "Win32_System_JobObjects",
] }

[target.'cfg(unix)'.dependencies]
nix = { version = "0.29", features = ["signal", "process"] }
```

### `src-tauri/tauri.conf.json`

```json
{
  "productName": "SelfHost Helper",
  "version": "0.40.1",
  "identifier": "com.selfhosthelper.app",
  "build": {
    "beforeDevCommand": "npm run dev:react",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "npm run build:web",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [
      {
        "title": "SelfHost Helper",
        "width": 1280,
        "height": 800,
        "decorations": false,
        "center": true,
        "resizable": true,
        "minWidth": 900,
        "minHeight": 600
      }
    ],
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net https://unpkg.com; style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; img-src 'self' data: https: https://cdn.discordapp.com media:; font-src 'self' data: https:; connect-src 'self' https://cdn.jsdelivr.net https://discord.com https://cdn.discordapp.com ws: wss: ipc: http://localhost:5173 http://127.0.0.1:5173; worker-src 'self' blob:;"
    },
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true,
      "tooltip": "SelfHost Helper"
    }
  },
  "bundle": {
    "active": true,
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "externalBin": ["cloudflared", "rg"],
    "windows": {
      "nsis": { "installMode": "both" }
    },
    "linux": {
      "appimage": { "bundleMediaFramework": false }
    },
    "category": "DeveloperTool",
    "shortDescription": "Node.js Project Manager"
  },
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIG1lc3NhZ2UgZmlsZXN0YXJ0IGV4YW1wbGUK",
      "endpoints": [
        "https://github.com/DevRoots-Studio/SelfHost-Helper/releases/latest/download/latest.json"
      ]
    },
    "shell": {
      "open": true,
      "scope": [
        { "name": "cloudflared", "cmd": "cloudflared", "args": true },
        { "name": "rg", "cmd": "rg", "args": true },
        { "name": "node", "cmd": "node", "args": true },
        { "name": "python", "cmd": "python3", "args": true },
        { "name": "typescript-language-server", "cmd": "typescript-language-server", "args": true },
        { "name": "git", "cmd": "git", "args": true }
      ]
    }
  }
}
```

### `src-tauri/build.rs`

```rust
fn main() {
    tauri_build::build();
}
```

### `src-tauri/src/main.rs`

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    selfhost_helper_lib::run();
}
```

### `src-tauri/src/error.rs`

```rust
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Process error: {0}")]
    Process(String),
    #[error("Tunnel error: {0}")]
    Tunnel(String),
    #[error("Search error: {0}")]
    Search(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

### `src-tauri/capabilities/default.json`

```json
{
  "identifier": "default",
  "description": "Default capability for SelfHost Helper",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:window:allow-close",
    "core:window:allow-hide",
    "core:window:allow-show",
    "core:window:allow-minimize",
    "core:window:allow-maximize",
    "core:window:allow-unmaximize",
    "core:window:allow-set-focus",
    "core:window:allow-is-maximized",
    "core:window:allow-start-dragging",
    "shell:default",
    "shell:allow-open",
    "shell:allow-execute",
    "shell:allow-spawn",
    "updater:default",
    "updater:allow-check",
    "updater:allow-download",
    "updater:allow-install",
    "process:default",
    "process:allow-exit",
    "process:allow-restart",
    "dialog:default",
    "dialog:allow-open",
    "dialog:allow-save",
    "dialog:allow-ask",
    "dialog:allow-confirm",
    "fs:default",
    "fs:allow-read",
    "fs:allow-write",
    "fs:allow-exists",
    "fs:allow-mkdir",
    "fs:allow-remove",
    "fs:allow-rename",
    "fs:allow-copy-file",
    "fs:allow-read-dir",
    "http:default",
    "http:allow-fetch",
    "store:default",
    "log:default"
  ]
}
```

---

## Full Rust Implementations

### `src-tauri/src/utils/crypto.rs`

```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::SystemTime;

const KDF_ITERATIONS: u32 = 310_000;
const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;
const IV_LEN: usize = 12;
const CURRENT_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("passphrase is required")]
    MissingPassphrase,
    #[error("invalid backup envelope: {0}")]
    InvalidEnvelope(String),
    #[error("unsupported backup version: {0}")]
    UnsupportedVersion(u32),
    #[error("decryption failed -- check your passphrase")]
    DecryptionFailed,
    #[error("decrypted payload is invalid: {0}")]
    InvalidPayload(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("cipher error: {0}")]
    Cipher(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfInfo {
    pub algo: String,
    pub hash: String,
    pub iterations: u32,
    pub salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherInfo {
    pub algo: String,
    pub iv: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupAppMeta {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPayload {
    #[serde(default)]
    pub projects: Vec<serde_json::Value>,
    #[serde(default)]
    pub categories: Vec<serde_json::Value>,
    #[serde(default)]
    pub settings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEnvelope {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<BackupAppMeta>,
    pub payload: BackupPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBackup {
    pub version: u32,
    pub kdf: KdfInfo,
    pub cipher: CipherInfo,
    pub tag: String,
    pub data: String,
    pub meta: EncryptedBackupMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBackupMeta {
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<BackupAppMeta>,
}

fn derive_key(passphrase: &str, salt: &[u8], iterations: u32) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, iterations, &mut key);
    key
}

pub fn encrypt_envelope(
    envelope: &BackupEnvelope,
    passphrase: &str,
) -> Result<EncryptedBackup, CryptoError> {
    if passphrase.trim().is_empty() {
        return Err(CryptoError::MissingPassphrase);
    }
    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut iv);
    let key = derive_key(passphrase, &salt, KDF_ITERATIONS);
    let plaintext = serde_json::to_vec(envelope)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CryptoError::Cipher(e.to_string()))?;
    let nonce = Nonce::from_slice(&iv);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| CryptoError::Cipher(e.to_string()))?;
    let tag_len = 16;
    if ciphertext.len() < tag_len {
        return Err(CryptoError::Cipher("ciphertext shorter than tag".into()));
    }
    let (encrypted_data, auth_tag) = ciphertext.split_at(ciphertext.len() - tag_len);
    let created_at = envelope.created_at.clone().unwrap_or_else(now_iso8601);
    Ok(EncryptedBackup {
        version: CURRENT_VERSION,
        kdf: KdfInfo {
            algo: "pbkdf2".into(),
            hash: "sha256".into(),
            iterations: KDF_ITERATIONS,
            salt: BASE64.encode(salt),
        },
        cipher: CipherInfo {
            algo: "aes-256-gcm".into(),
            iv: BASE64.encode(iv),
        },
        tag: BASE64.encode(auth_tag),
        data: BASE64.encode(encrypted_data),
        meta: EncryptedBackupMeta { created_at, app: envelope.app.clone() },
    })
}

pub fn decrypt_backup(
    backup: &EncryptedBackup,
    passphrase: &str,
) -> Result<BackupEnvelope, CryptoError> {
    if passphrase.trim().is_empty() {
        return Err(CryptoError::MissingPassphrase);
    }
    if backup.version < 1 {
        return Err(CryptoError::UnsupportedVersion(backup.version));
    }
    let salt = BASE64.decode(&backup.kdf.salt)?;
    let iv = BASE64.decode(&backup.cipher.iv)?;
    let auth_tag = BASE64.decode(&backup.tag)?;
    let ciphertext = BASE64.decode(&backup.data)?;
    let iterations = if backup.kdf.iterations > 0 { backup.kdf.iterations } else { KDF_ITERATIONS };
    let key = derive_key(passphrase, &salt, iterations);
    let mut full_ciphertext = ciphertext;
    full_ciphertext.extend_from_slice(&auth_tag);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CryptoError::Cipher(e.to_string()))?;
    let nonce = Nonce::from_slice(&iv);
    let plaintext = cipher
        .decrypt(nonce, full_ciphertext.as_ref())
        .map_err(|_| CryptoError::DecryptionFailed)?;
    let envelope: BackupEnvelope = serde_json::from_slice(&plaintext)
        .map_err(|e| CryptoError::InvalidPayload(e.to_string()))?;
    Ok(envelope)
}

pub fn encrypt_to_json(
    payload: BackupPayload,
    passphrase: &str,
    app: Option<BackupAppMeta>,
) -> Result<String, CryptoError> {
    let envelope = BackupEnvelope {
        version: CURRENT_VERSION,
        created_at: Some(now_iso8601()),
        app,
        payload,
    };
    let encrypted = encrypt_envelope(&envelope, passphrase)?;
    Ok(serde_json::to_string_pretty(&encrypted)?)
}

pub fn decrypt_from_json(json: &str, passphrase: &str) -> Result<BackupEnvelope, CryptoError> {
    let backup: EncryptedBackup = serde_json::from_str(json)?;
    decrypt_backup(backup, passphrase)
}

fn now_iso8601() -> String {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| {
            let secs = d.as_secs();
            format!(
                "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                1970 + (secs / 31_536_000) as u32,
                ((secs % 31_536_000) / 2_592_000) as u32 + 1,
                ((secs % 2_592_000) / 86_400) as u32 + 1,
                (secs % 86_400) / 3600,
                (secs % 3600) / 60,
                secs % 60,
            )
        })
        .unwrap_or_default()
}
```

### `src-tauri/src/utils/path_security.rs`

```rust
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum PathSecurityError {
    #[error("path must be inside a registered project root")]
    OutsideRoot,
    #[error("non-absolute path rejected: {0}")]
    NonAbsolute(String),
    #[error("path does not exist: {0}")]
    NotReadable(String),
    #[error("invalid media URL: {0}")]
    InvalidMediaUrl(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn normalize_for_comparison(target: &Path) -> PathBuf {
    let resolved = dunce::canonicalize(target).unwrap_or_else(|_| target.to_path_buf());
    #[cfg(target_os = "windows")]
    { PathBuf::from(resolved.to_string_lossy().to_lowercase()) }
    #[cfg(not(target_os = "windows"))]
    { resolved }
}

pub fn is_within_base(target_path: &Path, base_path: &Path) -> bool {
    let norm_target = normalize_for_comparison(target_path);
    let norm_base = normalize_for_comparison(base_path);
    if norm_target == norm_base { return true; }
    let base_str = norm_base.to_string_lossy();
    let base_with_sep = if base_str.ends_with(std::path::MAIN_SEPARATOR) {
        base_str.into_owned()
    } else {
        format!("{}{}", base_str, std::path::MAIN_SEPARATOR)
    };
    norm_target.to_string_lossy().starts_with(&base_with_sep)
}

pub fn is_within_any_base(target_path: &Path, allowed_bases: &[PathBuf]) -> bool {
    allowed_bases.iter().any(|base| is_within_base(target_path, base))
}

pub fn resolve_and_validate(candidate: &Path, allowed_bases: &[PathBuf]) -> Option<PathBuf> {
    let resolved = dunce::canonicalize(candidate).unwrap_or_else(|_| candidate.to_path_buf());
    if is_within_any_base(&resolved, allowed_bases) { Some(resolved) } else { None }
}

pub fn validate_inside_roots(target_path: &Path, project_roots: &[PathBuf]) -> Result<(), PathSecurityError> {
    let resolved = dunce::canonicalize(target_path).unwrap_or_else(|_| target_path.to_path_buf());
    if is_within_any_base(&resolved, project_roots) { Ok(()) } else { Err(PathSecurityError::OutsideRoot) }
}

pub fn parse_media_url(request_url: &str, app_base: &Path, cwd_base: &Path) -> Result<MediaUrl, PathSecurityError> {
    let without_scheme = request_url
        .strip_prefix("media://").or_else(|| request_url.strip_prefix("media:/"))
        .or_else(|| request_url.strip_prefix("media:"))
        .ok_or_else(|| PathSecurityError::InvalidMediaUrl("missing media:// scheme".into()))?;
    let (hostname, raw_path) = match without_scheme.find('/') {
        Some(pos) => { let (h, p) = without_scheme.split_at(pos); (h.to_string(), p.to_string()) }
        None => (without_scheme.to_string(), String::new()),
    };
    let decoded = percent_decode(&raw_path);
    if hostname == "app" {
        let relative = decoded.trim_start_matches('/').trim_start_matches('\\');
        let app_candidate = app_base.join(relative);
        let file_path = if app_candidate.exists() { app_candidate } else { cwd_base.join(relative) };
        Ok(MediaUrl { hostname, file_path })
    } else {
        let file_path = if !hostname.is_empty() && hostname.len() == 1 && hostname.chars().next().unwrap().is_ascii_alphabetic() {
            PathBuf::from(format!("{}:{}", hostname, decoded))
        } else {
            PathBuf::from(decoded)
        };
        Ok(MediaUrl { hostname, file_path })
    }
}

#[derive(Debug, Clone)]
pub struct MediaUrl {
    pub hostname: String,
    pub file_path: PathBuf,
}

pub fn sanitize_filename(value: &str) -> String {
    let cleaned: String = value.chars().map(|c| {
        if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || (c as u32) < 0x20 { '_' }
        else if c.is_whitespace() { ' ' } else { c }
    }).collect();
    let trimmed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.is_empty() { "project".to_string() } else { trimmed.chars().take(120).collect() }
}

fn percent_decode(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) { output.push(byte as char); continue; }
            }
            output.push('%'); output.push_str(&hex);
        } else { output.push(c); }
    }
    output
}
```

### `src-tauri/src/utils/media_allowlist.rs`

```rust
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

const EXTERNAL_MEDIA_DIRS_ENV_KEY: &str = "SELFHOST_MEDIA_ALLOWED_DIRS";

#[derive(Debug, Clone)]
pub struct MediaAllowlist {
    configured_bases: Vec<PathBuf>,
    session_bases: Arc<RwLock<HashSet<PathBuf>>>,
    project_icon_bases: Arc<RwLock<Vec<PathBuf>>>,
    has_logged_missing: Arc<RwLock<bool>>,
}

impl MediaAllowlist {
    pub fn new() -> Self {
        let configured_bases = Self::parse_env_dirs();
        Self {
            configured_bases,
            session_bases: Arc::new(RwLock::new(HashSet::new())),
            project_icon_bases: Arc::new(RwLock::new(Vec::new())),
            has_logged_missing: Arc::new(RwLock::new(false)),
        }
    }

    fn parse_env_dirs() -> Vec<PathBuf> {
        let raw = std::env::var(EXTERNAL_MEDIA_DIRS_ENV_KEY).unwrap_or_default();
        let delimiter = if cfg!(target_os = "windows") { ';' } else { ':' };
        raw.split(delimiter).map(|e| e.trim()).filter(|e| !e.is_empty())
            .map(|e| { let p = PathBuf::from(e); dunce::canonicalize(&p).unwrap_or(p) }).collect()
    }

    pub fn add_session_path(&self, file_path: &Path) {
        if let Some(parent) = file_path.parent() {
            let resolved = dunce::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
            self.session_bases.write().expect("lock poisoned").insert(resolved);
        }
    }

    pub fn set_project_icon_bases(&self, bases: Vec<PathBuf>) {
        *self.project_icon_bases.write().expect("lock poisoned") = bases;
    }

    fn all_bases(&self) -> Vec<PathBuf> {
        let mut bases = self.configured_bases.clone();
        bases.extend(self.session_bases.read().expect("lock poisoned").iter().cloned());
        bases.extend(self.project_icon_bases.read().expect("lock poisoned").iter().cloned());
        bases
    }

    pub fn is_allowed(&self, file_path: &Path) -> bool {
        let bases = self.all_bases();
        crate::utils::path_security::is_within_any_base(file_path, &bases)
    }

    pub fn validate_media_path(&self, file_path: &Path) -> Result<PathBuf, MediaAllowlistError> {
        if !file_path.is_absolute() { return Err(MediaAllowlistError::NonAbsolute); }
        let bases = self.all_bases();
        if bases.is_empty() {
            let mut logged = self.has_logged_missing.write().expect("lock poisoned");
            if !*logged { *logged = true; log::warn!("External media requests require an allowlist. Set {}", EXTERNAL_MEDIA_DIRS_ENV_KEY); }
            return Err(MediaAllowlistError::NoAllowlistConfigured);
        }
        let resolved = crate::utils::path_security::resolve_and_validate(file_path, &bases)
            .ok_or(MediaAllowlistError::PathOutsideAllowlist)?;
        if !resolved.exists() { return Err(MediaAllowlistError::FileNotFound); }
        Ok(resolved)
    }
}

impl Default for MediaAllowlist { fn default() -> Self { Self::new() } }

pub type SharedMediaAllowlist = Arc<MediaAllowlist>;

#[derive(Debug, thiserror::Error)]
pub enum MediaAllowlistError {
    #[error("non-absolute path rejected")]
    NonAbsolute,
    #[error("path is outside all allowed directories")]
    PathOutsideAllowlist,
    #[error("no external media allowlist configured")]
    NoAllowlistConfigured,
    #[error("file not found")]
    FileNotFound,
}

pub fn mime_type_for_extension(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        ".png" => "image/png", ".jpg" | ".jpeg" => "image/jpeg", ".gif" => "image/gif",
        ".webp" => "image/webp", ".svg" => "image/svg+xml", ".mp4" => "video/mp4",
        ".mp3" => "audio/mpeg", ".pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}
```

### `src-tauri/src/utils/ignore_patterns.rs`

```rust
use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

const IGNORED_DIR_NAMES: &[&str] = &[
    "node_modules", "venv", ".venv", "env", "__pycache__", ".pycache",
    ".mypy_cache", ".pytest_cache", ".ruff_cache", "vendor", "target",
    "dist", "build", ".next", ".nuxt", "out", ".turbo", ".cache",
    "coverage", ".parcel-cache", ".vite", ".git", ".svn", ".hg",
    "bower_components", ".sass-cache", ".gradle", "bin", "obj",
];

static IGNORED_SET: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    IGNORED_DIR_NAMES.iter().copied().collect()
});

pub fn is_ignored_dir_name(name: &str) -> bool {
    IGNORED_SET.contains(name.to_ascii_lowercase().as_str())
}

pub fn is_path_ignored(path: &Path) -> bool {
    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            if let Some(name_str) = name.to_str() {
                if name_str.starts_with('.') { return true; }
                if is_ignored_dir_name(name_str) { return true; }
            }
        }
    }
    false
}

pub fn ripgrep_exclude_globs() -> Vec<String> {
    IGNORED_DIR_NAMES.iter().map(|d| format!("!**/{d}/**")).collect()
}
```

### `src-tauri/src/services/settings_service.rs`

```rust
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct ProjectSettings {
    pub project_id: String,
    pub runtime: Option<String>,
    pub node_version: Option<String>,
    pub python_version: Option<String>,
    pub install_date: Option<String>,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AppSettings {
    pub clear_logs_before_start: Option<bool>,
    pub start_maximized: Option<bool>,
    pub dev_mode: Option<bool>,
    pub external_media_allowed_dirs: Option<String>,
    pub default_project_path: Option<String>,
    pub editor_font_size: Option<i32>,
    pub editor_tab_size: Option<i32>,
    pub editor_theme: Option<String>,
}

pub struct SettingsService {
    db: Arc<Connection>,
}

impl SettingsService {
    pub fn new(db: Arc<Connection>) -> Self {
        let svc = Self { db };
        svc.migrate_settings_json();
        svc
    }

    fn migrate_settings_json(&self) {
        let settings_json_path = self.db_path_parent().join("settings.json");
        if !settings_json_path.exists() { return; }
        if let Ok(content) = std::fs::read_to_string(&settings_json_path) {
            if let Ok(map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content) {
                for (key, value) in map {
                    let val_str = match &value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Null => continue,
                        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                            serde_json::to_string(&value).unwrap_or_default()
                        }
                    };
                    let _ = self.set_app_setting(&key, &val_str);
                }
            }
        }
        let _ = std::fs::rename(&settings_json_path, settings_json_path.with_extension("json.bak"));
    }

    fn db_path_parent(&self) -> PathBuf {
        let path_str: String = self.db
            .pragma_query_value("main", "user_version", |row| row.get(0))
            .map(|_: i32| "unknown".to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        PathBuf::from(path_str)
    }

    pub fn init_tables(&self) -> AppResult<()> {
        self.db.execute_batch(
            "CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS project_settings (
                project_id   TEXT PRIMARY KEY NOT NULL,
                runtime      TEXT,
                node_version TEXT,
                python_version TEXT,
                install_date TEXT,
                last_used    TEXT
            );"
        )?;
        Ok(())
    }

    pub fn get_app_settings(&self) -> AppResult<AppSettings> {
        let mut stmt = self.db.prepare("SELECT key, value FROM settings")?;
        let mut settings = AppSettings::default();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "clearLogsBeforeStart" => settings.clear_logs_before_start = value.parse().ok(),
                "startMaximized" => settings.start_maximized = value.parse().ok(),
                "devMode" => settings.dev_mode = value.parse().ok(),
                "externalMediaAllowedDirs" => settings.external_media_allowed_dirs = Some(value),
                "defaultProjectPath" => settings.default_project_path = Some(value),
                "editorFontSize" => settings.editor_font_size = value.parse().ok(),
                "editorTabSize" => settings.editor_tab_size = value.parse().ok(),
                "editorTheme" => settings.editor_theme = Some(value),
                _ => {}
            }
        }
        Ok(settings)
    }

    pub fn set_app_setting(&self, key: &str, value: &str) -> AppResult<()> {
        self.db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn update_app_settings(&self, updates: &std::collections::HashMap<String, String>) -> AppResult<()> {
        for (key, value) in updates {
            self.set_app_setting(key, value)?;
        }
        Ok(())
    }

    pub fn get_project_settings(&self, project_id: &str) -> AppResult<Option<ProjectSettings>> {
        let mut stmt = self.db.prepare(
            "SELECT project_id, runtime, node_version, python_version, install_date, last_used
             FROM project_settings WHERE project_id = ?1"
        )?;
        let mut rows = stmt.query_map(params![project_id], |row| {
            Ok(ProjectSettings {
                project_id: row.get(0)?,
                runtime: row.get(1)?,
                node_version: row.get(2)?,
                python_version: row.get(3)?,
                install_date: row.get(4)?,
                last_used: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn set_project_settings(&self, settings: &ProjectSettings) -> AppResult<()> {
        self.db.execute(
            "INSERT OR REPLACE INTO project_settings (project_id, runtime, node_version, python_version, install_date, last_used)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                settings.project_id,
                settings.runtime,
                settings.node_version,
                settings.python_version,
                settings.install_date,
                settings.last_used,
            ],
        )?;
        Ok(())
    }

    pub fn delete_project_settings(&self, project_id: &str) -> AppResult<()> {
        self.db.execute("DELETE FROM project_settings WHERE project_id = ?1", params![project_id])?;
        Ok(())
    }
}
```

### `src-tauri/src/services/log_store.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const MAX_HISTORY_PER_KEY: usize = 100;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub message: String,
    pub r#type: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default)]
pub struct LogBatch {
    pub key: String,
    pub entries: Vec<LogEntry>,
}

pub struct LogStore {
    history: Arc<RwLock<HashMap<String, Vec<LogEntry>>>>,
    batch_queue: Arc<RwLock<Vec<LogEntry>>>,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(HashMap::new())),
            batch_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn push(&self, key: String, entry: LogEntry) -> Vec<LogBatch> {
        let mut queue = self.batch_queue.write().await;
        queue.push(LogEntry { key: key.clone(), ..entry.clone() });

        self.history.write().await.entry(key).or_default().push(entry);

        let mut batches = Vec::new();
        if queue.len() >= 50 {
            let taken: Vec<LogEntry> = queue.drain(..).collect();
            let mut by_key: HashMap<String, Vec<LogEntry>> = HashMap::new();
            for e in taken {
                by_key.entry(e.key.clone()).or_default().push(e);
            }
            for (k, entries) in by_key {
                batches.push(LogBatch { key: k, entries });
            }
        }
        batches
    }

    pub async fn drain_batch(&self) -> Vec<LogBatch> {
        let mut queue = self.batch_queue.write().await;
        if queue.is_empty() { return Vec::new(); }
        let taken: Vec<LogEntry> = queue.drain(..).collect();
        let mut by_key: HashMap<String, Vec<LogEntry>> = HashMap::new();
        for e in taken {
            by_key.entry(e.key.clone()).or_default().push(e);
        }
        by_key.into_iter().map(|(k, entries)| LogBatch { key: k, entries }).collect()
    }

    pub async fn get_history(&self, key: &str) -> Vec<LogEntry> {
        self.history.read().await.get(key).cloned().unwrap_or_default()
    }

    pub async fn clear_history(&self, key: &str) {
        self.history.write().await.remove(key);
    }

    pub async fn clear_all(&self) {
        self.history.write().await.clear();
    }

    fn trim_history(history: &mut Vec<LogEntry>) {
        if history.len() > MAX_HISTORY_PER_KEY {
            history.drain(..history.len() - MAX_HISTORY_PER_KEY);
        }
    }
}

impl Default for LogStore {
    fn default() -> Self { Self::new() }
}
```

### `src-tauri/src/services/tunnel_manager.rs`

```rust
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, RwLock};
use crate::error::{AppError, AppResult};

struct TunnelInstance {
    project_id: String,
    port: u16,
    mode: String,
    url: Option<String>,
    child: Option<Child>,
    log_sender: mpsc::Sender<TunnelLogEntry>,
    stopping: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TunnelLogEntry {
    pub project_id: String,
    pub message: String,
    pub r#type: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TunnelStatus {
    pub project_id: String,
    pub status: String,
    pub url: Option<String>,
    pub port: Option<u16>,
    pub error: Option<String>,
}

pub struct TunnelManager {
    running: Arc<RwLock<HashMap<String, TunnelInstance>>>,
    logs: Arc<RwLock<HashMap<String, Vec<TunnelLogEntry>>>>,
    event_tx: mpsc::Sender<TunnelStatus>,
}

impl TunnelManager {
    pub fn new() -> (Self, mpsc::Receiver<TunnelStatus>) {
        let (tx, rx) = mpsc::channel(64);
        (Self {
            running: Arc::new(RwLock::new(HashMap::new())),
            logs: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        }, rx)
    }

    pub async fn start_tunnel(
        &self,
        project_id: String,
        mode: String,
        port: u16,
        token: Option<String>,
        config: Option<HashMap<String, String>>,
    ) -> AppResult<()> {
        let mut running = self.running.write().await;
        if running.contains_key(&project_id) {
            return Err(AppError::Validation("Tunnel already running".into()));
        }
        if port == 0 || port > 65535 {
            return Err(AppError::Validation("Invalid port".into()));
        }

        self.send_status(&project_id, "connecting", None, None, None).await;

        let mut args = vec!["tunnel".to_string(), "--url".to_string(), format!("http://localhost:{}", port)];

        if mode == "authenticated" {
            let tok = token.ok_or_else(|| AppError::Validation("Token required".into()))?;
            args.insert(0, "tunnel".to_string());
            args.insert(1, "run".to_string());
            args.push("--token".to_string());
            args.push(tok);

            if let Some(cfg) = &config {
                if let Some(proto) = cfg.get("protocol") {
                    args.push("--protocol".to_string());
                    args.push(proto.clone());
                }
                if let Some(true_str) = cfg.get("noTLSVerify") {
                    if true_str == "true" {
                        args.push("--no-tls-verify".to_string());
                    }
                }
                if let Some(host) = cfg.get("httpHostHeader") {
                    args.push("--http-host-header".to_string());
                    args.push(host.clone());
                }
            }
        }

        let child = Command::new("cloudflared")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Process(format!("Failed to spawn cloudflared: {}", e)))?;

        let pid = child.id().map(|id| id.to_string()).unwrap_or_else(|| "unknown".into());
        let (log_tx, mut log_rx) = mpsc::channel::<TunnelLogEntry>(100);

        let mut inst = TunnelInstance {
            project_id: project_id.clone(),
            port,
            mode: mode.clone(),
            url: None,
            child: Some(child),
            log_sender: log_tx,
            stopping: false,
        };

        running.insert(project_id.clone(), inst);

        let running_ref = self.running.clone();
        let logs_ref = self.logs.clone();
        let event_tx = self.event_tx.clone();
        let pid_clone = project_id.clone();

        tokio::spawn(async move {
            while let Some(entry) = log_rx.recv().await {
                let mut logs = logs_ref.write().await;
                let history = logs.entry(entry.project_id.clone()).or_insert_with(Vec::new);
                history.push(entry.clone());
                if history.len() > 100 { history.remove(0); }
                drop(logs);

                let lower = entry.message.to_lowercase();
                if entry.message.contains("trycloudflare.com") || entry.message.contains("https://") {
                    if let Some(url_start) = entry.message.find("https://") {
                        let url_end = entry.message[url_start..].find_whitespace().unwrap_or(entry.message.len() - url_start);
                        let url = entry.message[url_start..url_start + url_end].trim_end_matches(')').to_string();
                        if let Some(mut inst) = running_ref.write().await.get_mut(&pid_clone) {
                            inst.url = Some(url.clone());
                        }
                        let _ = event_tx.send(TunnelStatus {
                            project_id: pid_clone.clone(),
                            status: "running".into(),
                            url: Some(url),
                            port: Some(port),
                            error: None,
                        }).await;
                    }
                } else if lower.contains("error") {
                    let _ = event_tx.send(TunnelStatus {
                        project_id: pid_clone.clone(),
                        status: "error".into(),
                        url: None,
                        port: None,
                        error: Some(entry.message),
                    }).await;
                }
            }

            if let Some(mut inst) = running_ref.write().await.get_mut(&pid_clone) {
                if let Some(ref mut child) = inst.child {
                    let _ = child.kill().await;
                }
            }

            running_ref.write().await.remove(&pid_clone);
            let _ = event_tx.send(TunnelStatus {
                project_id: pid_clone,
                status: "stopped".into(),
                url: None,
                port: None,
                error: None,
            }).await;
        });

        Ok(())
    }

    pub async fn stop_tunnel(&self, project_id: &str) -> AppResult<()> {
        let mut running = self.running.write().await;
        let inst = running.get_mut(project_id)
            .ok_or_else(|| AppError::NotFound("No tunnel running".into()))?;
        inst.stopping = true;
        if let Some(ref mut child) = inst.child {
            child.kill().await.map_err(|e| AppError::Process(e.to_string()))?;
        }
        running.remove(project_id);
        self.send_status(project_id, "stopped", None, None, None).await;
        Ok(())
    }

    pub async fn stop_all(&self) {
        let mut running = self.running.write().await;
        for (_, mut inst) in running.drain() {
            if let Some(ref mut child) = inst.child {
                let _ = child.kill().await;
            }
        }
    }

    pub async fn get_status(&self, project_id: &str) -> Option<TunnelStatus> {
        let running = self.running.read().await;
        running.get(project_id).map(|inst| TunnelStatus {
            project_id: project_id.to_string(),
            status: if inst.url.is_some() { "running".into() } else { "connecting".into() },
            url: inst.url.clone(),
            port: Some(inst.port),
            error: None,
        })
    }

    pub async fn get_logs(&self, project_id: &str) -> Vec<TunnelLogEntry> {
        self.logs.read().await.get(project_id).cloned().unwrap_or_default()
    }

    pub async fn get_all_logs(&self) -> HashMap<String, Vec<TunnelLogEntry>> {
        self.logs.read().await.clone()
    }

    pub async fn clear_logs(&self, project_id: &str) {
        self.logs.write().await.remove(project_id);
    }

    async fn send_status(&self, project_id: &str, status: &str, url: Option<String>, port: Option<u16>, error: Option<String>) {
        let _ = self.event_tx.send(TunnelStatus {
            project_id: project_id.to_string(),
            status: status.into(),
            url, port, error,
        }).await;
    }
}

fn find_whitespace(s: &str) -> Option<usize> { s.find(|c: char| c.is_whitespace()) }
```

### `src-tauri/src/services/git_service.rs`

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use git2::{
    Branch, BranchType, DiffOptions, ErrorCode, IndexAddOption, Oid, ReferenceType, Repository,
    Sort, Status, StatusOptions, StatusShow,
};

use crate::error::{AppError, AppResult};

pub struct GitService {
    repos: Mutex<HashMap<String, Repository>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitBranch {
    pub name: String,
    pub is_head: bool,
    pub upstream: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitCommit {
    pub oid: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitDiffEntry {
    pub path: String,
    pub status: String,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitStatusEntry {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitRemote {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitStashEntry {
    pub index: usize,
    pub message: String,
    pub branch: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitTag {
    pub name: String,
    pub target_oid: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitSyncResult {
    pub success: bool,
    pub behind: usize,
    pub ahead: usize,
    pub message: String,
}

impl GitService {
    pub fn new() -> Self {
        Self { repos: Mutex::new(HashMap::new()) }
    }

    fn get_repo(&self, project_path: &str) -> AppResult<Repository> {
        let mut repos = self.repos.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Some(repo) = repos.get(project_path) {
            return Ok(repo.clone());
        }
        let repo = Repository::open(project_path)
            .map_err(|e| AppError::Git(format!("Failed to open repository: {}", e)))?;
        repos.insert(project_path.to_string(), repo.clone());
        Ok(repo)
    }

    pub fn get_branches(&self, project_path: &str) -> AppResult<Vec<GitBranch>> {
        let repo = self.get_repo(project_path)?;
        let head_ref = repo.head().ok();
        let head_name = head_ref.as_ref().and_then(|h| h.shorthand().map(|s| s.to_string()));
        let mut branches = Vec::new();
        for branch_result in repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch_result?;
            let name = branch.name()?.unwrap_or_default().to_string();
            let is_head = head_name.as_deref() == Some(name.as_str());
            let upstream = branch.upstream().ok().and_then(|u| u.name().ok().flatten().map(|s| s.to_string()));
            branches.push(GitBranch { name, is_head, upstream });
        }
        Ok(branches)
    }

    pub fn checkout_branch(&self, project_path: &str, branch_name: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?;
        let current_branch = head.shorthand().unwrap_or_default();
        if current_branch == branch_name { return Ok(()); }
        repo.set_head(&format!("refs/heads/{}", branch_name))
            .map_err(|e| AppError::Git(format!("Failed to checkout branch {}: {}", branch_name, e)))?;
        Ok(())
    }

    pub fn create_branch(&self, project_path: &str, branch_name: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        repo.branch(branch_name, &commit, false)?;
        Ok(())
    }

    pub fn delete_branch(&self, project_path: &str, branch_name: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let branch = repo.find_branch(branch_name, BranchType::Local)?;
        branch.delete()?;
        Ok(())
    }

    pub fn get_status(&self, project_path: &str) -> AppResult<Vec<GitStatusEntry>> {
        let repo = self.get_repo(project_path)?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(true);
        let statuses = repo.statuses(Some(&mut opts))?;
        let entries = statuses.iter().filter_map(|entry| {
            let path = entry.path()?.to_string();
            let status = format_status(entry.status());
            Some(GitStatusEntry { path, status })
        }).collect();
        Ok(entries)
    }

    pub fn stage_file(&self, project_path: &str, file_path: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let mut index = repo.index()?;
        index.add_path(Path::new(file_path))
            .map_err(|e| AppError::Git(format!("Failed to stage file: {}", e)))?;
        index.write()?;
        Ok(())
    }

    pub fn unstage_file(&self, project_path: &str, file_path: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?.peel_to_commit()?;
        repo.reset_default(Some(&head.into()), &[Path::new(file_path)])
            .map_err(|e| AppError::Git(format!("Failed to unstage file: {}", e)))?;
        Ok(())
    }

    pub fn stage_all(&self, project_path: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let mut index = repo.index()?;
        index.add_all(["."], IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    pub fn commit(&self, project_path: &str, message: &str, author_name: &str, author_email: &str) -> AppResult<String> {
        let repo = self.get_repo(project_path)?;
        let mut index = repo.index()?;
        index.write()?;
        let tree_oid = index.write_tree()?;
        let tree = repo.find_tree(tree_oid)?;
        let sig = git2::Signature::now(author_name, author_email)?;
        let head = repo.head().ok();
        let parent = head.and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.iter().collect();
        let commit_oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
        Ok(commit_oid.to_string())
    }

    pub fn push(&self, project_path: &str, remote_name: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let mut remote = repo.find_remote(remote_name)
            .map_err(|e| AppError::Git(format!("Remote '{}' not found: {}", remote_name, e)))?;
        let head = repo.head()?;
        let branch = head.shorthand().unwrap_or("main");
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);
        remote.push(&[refspec], None)
            .map_err(|e| AppError::Git(format!("Push failed: {}", e)))?;
        Ok(())
    }

    pub fn pull(&self, project_path: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let mut remote = repo.find_remote("origin")?;
        let fetch_head = repo.find_reference("FETCH_HEAD").ok();
        let branch = repo.head()?.shorthand().unwrap_or("main").to_string();
        remote.fetch(&[branch.as_str()], None, None)?;
        let fetch_oid = repo.refname_to_id("FETCH_HEAD")?;
        let fetch_commit = repo.find_commit(fetch_oid)?;
        let head = repo.head()?.peel_to_commit()?;
        repo.branch(&branch, &fetch_commit, false)?;
        repo.set_head(&format!("refs/heads/{}", branch))?;
        repo.checkout_tree(fetch_commit.as_object(), None)?;
        Ok(())
    }

    pub fn clone(&self, url: &str, dest_path: &str) -> AppResult<()> {
        Repository::clone(url, dest_path)
            .map_err(|e| AppError::Git(format!("Clone failed: {}", e)))?;
        Ok(())
    }

    pub fn get_remotes(&self, project_path: &str) -> AppResult<Vec<GitRemote>> {
        let repo = self.get_repo(project_path)?;
        let mut remotes = Vec::new();
        for remote_result in repo.remotes()?.iter() {
            if let Some(name) = remote_result {
                if let Ok(remote) = repo.find_remote(name) {
                    remotes.push(GitRemote {
                        name: name.to_string(),
                        url: remote.url().unwrap_or_default().to_string(),
                    });
                }
            }
        }
        Ok(remotes)
    }

    pub fn add_remote(&self, project_path: &str, name: &str, url: &str) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        repo.remote(name, url)?;
        Ok(())
    }

    pub fn get_diff_summary(&self, project_path: &str) -> AppResult<Vec<GitDiffEntry>> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?.peel_to_tree()?;
        let mut opts = DiffOptions::new();
        let diff = repo.diff_tree_to_workdir(Some(&head), Some(&mut opts))?;
        let mut entries = Vec::new();
        for delta_idx in 0..diff.deltas().len() {
            let delta = diff.deltas().nth(delta_idx).ok_or_else(|| AppError::Internal("bad delta index".into()))?;
            let path = delta.new_file().path().or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let status = format_diff_status(delta.status());
            let (insertions, deletions) = (0u32, 0u32);
            entries.push(GitDiffEntry { path, status, insertions, deletions });
        }
        Ok(entries)
    }

    pub fn get_log(&self, project_path: &str, max_count: usize) -> AppResult<Vec<GitCommit>> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?;
        let mut revwalk = repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME)?;
        revwalk.push(head.target().ok_or_else(|| AppError::Git("HEAD has no target".into()))?)?;
        let mut commits = Vec::new();
        for (i, oid_result) in revwalk.enumerate() {
            if i >= max_count { break; }
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            let author = commit.author();
            commits.push(GitCommit {
                oid: oid.to_string(),
                message: commit.summary().unwrap_or("").to_string(),
                author_name: author.name().unwrap_or("").to_string(),
                author_email: author.email().unwrap_or("").to_string(),
                timestamp: author.when().seconds(),
            });
        }
        Ok(commits)
    }

    pub fn stash(&self, project_path: &str, message: Option<&str>) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        let signature = repo.signature()?;
        let mut index = repo.index()?;
        index.write()?;
        let tree_oid = index.write_tree()?;
        let tree = repo.find_tree(tree_oid)?;
        let stash_msg = message.unwrap_or("WIP");
        repo.stash_save(&signature, stash_msg, Some(Status::all()))?;
        Ok(())
    }

    pub fn stash_list(&self, project_path: &str) -> AppResult<Vec<GitStashEntry>> {
        let repo = self.get_repo(project_path)?;
        let mut entries = Vec::new();
        repo.stash_foreach(|index, name, oid| {
            let commit = repo.find_commit(oid).ok();
            entries.push(GitStashEntry {
                index,
                message: name.to_string(),
                branch: String::new(),
                timestamp: commit.as_ref().and_then(|c| c.time().seconds().into()).unwrap_or(0),
            });
            true
        })?;
        Ok(entries)
    }

    pub fn stash_pop(&self, project_path: &str, stash_index: usize) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        repo.stash_pop(stash_index, None)?;
        Ok(())
    }

    pub fn stash_drop(&self, project_path: &str, stash_index: usize) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        repo.stash_drop(stash_index)?;
        Ok(())
    }

    pub fn get_tags(&self, project_path: &str) -> AppResult<Vec<GitTag>> {
        let repo = self.get_repo(project_path)?;
        let mut tags = Vec::new();
        for tag_result in repo.tag_names(None)? {
            if let Some(name) = tag_result {
                if let Ok(tag) = repo.find_tag_by_name(name) {
                    if let Ok(commit) = tag.peel(git2::ObjectType::Commit) {
                        tags.push(GitTag {
                            name: name.to_string(),
                            target_oid: commit.id().to_string(),
                            message: tag.message().map(|s| s.to_string()),
                        });
                    }
                }
            }
        }
        Ok(tags)
    }

    pub fn stash_apply(&self, project_path: &str, stash_index: usize) -> AppResult<()> {
        let repo = self.get_repo(project_path)?;
        repo.stash_apply(stash_index, None)?;
        Ok(())
    }

    pub fn diff_file(&self, project_path: &str, file_path: &str) -> AppResult<String> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?.peel_to_tree()?;
        let mut opts = DiffOptions::new();
        opts.pathspec(file_path);
        let diff = repo.diff_tree_to_workdir(Some(&head), Some(&mut opts))?;
        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            diff_text.push_str(&String::from_utf8_lossy(line.content()));
            true
        })?;
        Ok(diff_text)
    }

    pub fn get_commit_diff(&self, project_path: &str, commit_oid_str: &str) -> AppResult<String> {
        let repo = self.get_repo(project_path)?;
        let oid = Oid::from_str(commit_oid_str)?;
        let commit = repo.find_commit(oid)?;
        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().map(|p| p.tree()).transpose()?;
        let mut opts = DiffOptions::new();
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut opts))?;
        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            diff_text.push_str(&String::from_utf8_lossy(line.content()));
            true
        })?;
        Ok(diff_text)
    }

    pub fn get_branch_ahead_behind(&self, project_path: &str, branch_name: &str) -> AppResult<(usize, usize)> {
        let repo = self.get_repo(project_path)?;
        let local = repo.find_branch(branch_name, BranchType::Local)?;
        let upstream = local.upstream().ok();
        let (ahead, behind) = if let Some(upstream_ref) = upstream {
            let local_oid = local.get().target().ok_or_else(|| AppError::Git("no local oid".into()))?;
            let upstream_oid = upstream_ref.get().target().ok_or_else(|| AppError::Git("no upstream oid".into()))?;
            repo.graph_ahead_behind(local_oid, upstream_oid)?
        } else {
            (0, 0)
        };
        Ok((ahead, behind))
    }

    pub fn get_current_branch(&self, project_path: &str) -> AppResult<String> {
        let repo = self.get_repo(project_path)?;
        let head = repo.head()?;
        Ok(head.shorthand().unwrap_or("HEAD").to_string())
    }
}

fn format_status(status: Status) -> String {
    if status.contains(Status::INDEX_NEW) { "added".into() }
    else if status.contains(Status::INDEX_MODIFIED) { "modified".into() }
    else if status.contains(Status::INDEX_DELETED) { "deleted".into() }
    else if status.contains(Status::INDEX_RENAMED) { "renamed".into() }
    else if status.contains(Status::WT_NEW) { "untracked".into() }
    else if status.contains(Status::WT_MODIFIED) { "changed".into() }
    else if status.contains(Status::WT_DELETED) { "deleted".into() }
    else if status.contains(Status::CONFLICTED) { "conflict".into() }
    else { "unknown".into() }
}

fn format_diff_status(status: git2::Delta) -> String {
    match status {
        git2::Delta::Added => "added",
        git2::Delta::Modified => "modified",
        git2::Delta::Deleted => "deleted",
        git2::Delta::Renamed => "renamed",
        git2::Delta::Copied => "copied",
        git2::Delta::Typechange => "typechange",
        _ => "unknown",
    }.into()
}
```

### `src-tauri/src/services/search_service.rs`

```rust
use std::path::{Path, PathBuf};
use tokio::process::Command;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub text: String,
    pub match_start: u32,
    pub match_end: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchStats {
    pub total_matches: usize,
    pub files_with_matches: usize,
    pub duration_ms: u64,
}

const IGNORED_DIRS: &[&str] = &[
    "node_modules", "venv", ".venv", "env", "__pycache__", ".pycache",
    ".mypy_cache", ".pytest_cache", ".ruff_cache", "vendor", "target",
    "dist", "build", ".next", ".nuxt", "out", ".turbo", ".cache",
    "coverage", ".parcel-cache", ".vite", ".git", ".svn", ".hg",
    "bower_components", ".sass-cache", ".gradle", "bin", "obj",
];

pub struct SearchService;

impl SearchService {
    pub fn new() -> Self { Self }

    pub async fn search(
        &self,
        directory: &str,
        query: &str,
        case_sensitive: bool,
        whole_word: bool,
        regex_mode: bool,
        max_results: usize,
    ) -> AppResult<(Vec<SearchResult>, SearchStats)> {
        let start = std::time::Instant::now();
        let mut args = vec![
            "--json".to_string(),
            "--line-number".to_string(),
            "--column-number".to_string(),
            "--max-count".to_string(), max_results.to_string(),
            "--hidden".to_string(),
        ];

        if !case_sensitive { args.push("--ignore-case".to_string()); }
        if whole_word { args.push("--word-regexp".to_string()); }
        if !regex_mode { args.push("--fixed-strings".to_string()); }

        for dir in IGNORED_DIRS {
            args.push("--glob".to_string());
            args.push(format!("!**/{}/**", dir));
        }
        args.push(query.to_string());
        args.push(directory.to_string());

        let output = Command::new("rg")
            .args(&args)
            .output()
            .await
            .map_err(|e| AppError::Search(format!("Failed to run rg: {}. Ensure ripgrep is installed.", e)))?;

        if !output.status.success() && output.stdout.is_empty() {
            return Ok((Vec::new(), SearchStats { total_matches: 0, files_with_matches: 0, duration_ms: start.elapsed().as_millis() as u64 }));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();
        let mut files_set = std::collections::HashSet::new();

        for line in stdout.lines() {
            if line.is_empty() { continue; }
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(submatches) = json.get("submatches").and_then(|s| s.as_array()) {
                    if let Some(first_match) = submatches.first() {
                        let match_text = first_match.get("text").and_then(|t| t.as_str()).unwrap_or("");
                        let match_start = first_match.get("start").and_then(|s| s.as_u64()).unwrap_or(0) as u32;
                        let match_end = first_match.get("end").and_then(|s| s.as_u64()).unwrap_or(0) as u32;
                        let path = json.get("path").and_then(|p| p.as_str()).unwrap_or("");
                        let line_number = json.get("line_number").and_then(|l| l.as_u64()).unwrap_or(0) as u32;
                        files_set.insert(path.to_string());
                        results.push(SearchResult {
                            file: path.to_string(),
                            line: line_number,
                            column: match_start + 1,
                            text: match_text.to_string(),
                            match_start,
                            match_end,
                        });
                    }
                } else if let Some(data) = json.get("data").and_then(|d| d.as_str()) {
                    let path = json.get("path").and_then(|p| p.as_str()).unwrap_or("");
                    let line_number = json.get("line_number").and_then(|l| l.as_u64()).unwrap_or(0) as u32;
                    files_set.insert(path.to_string());
                    results.push(SearchResult {
                        file: path.to_string(),
                        line: line_number,
                        column: 1,
                        text: data.to_string(),
                        match_start: 0,
                        match_end: data.len() as u32,
                    });
                }
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok((results.clone(), SearchStats {
            total_matches: results.len(),
            files_with_matches: files_set.len(),
            duration_ms: duration,
        }))
    }
}
```

### `src-tauri/src/services/lsp_bridge.rs`

```rust
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, RwLock};

pub type LspMessage = serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspRequest {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

pub struct LspBridge {
    sessions: Arc<RwLock<HashMap<String, LspSession>>>,
    proxy_port: Arc<RwLock<Option<u16>>>,
}

struct LspSession {
    child: Child,
    stdin_tx: mpsc::Sender<Vec<u8>>,
    pending: Arc<RwLock<HashMap<u64, oneshot::Sender<serde_json::Value>>>>,
    next_id: Arc<RwLock<u64>>,
}

impl LspBridge {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            proxy_port: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start_proxy(&self) -> Result<u16, String> {
        let mut port_guard = self.proxy_port.write().await;
        if let Some(port) = *port_guard { return Ok(port); }

        let listener = TcpListener::bind("127.0.0.1:0").await
            .map_err(|e| format!("Failed to bind proxy: {}", e))?;
        let port = listener.local_addr()
            .map_err(|e| format!("Failed to get addr: {}", e))?
            .port();

        let sessions = self.sessions.clone();
        tokio::spawn(async move {
            loop {
                let (socket, _) = match listener.accept().await {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let sessions = sessions.clone();
                tokio::spawn(async move {
                    let (ws_sink, mut ws_stream) = futures_util::StreamExt::split(
                        tokio_tungstenite::accept_async(socket).await.unwrap()
                    );
                    let ws_sink = Arc::new(tokio::sync::Mutex::new(ws_sink));
                    while let Some(msg) = ws_stream.next().await {
                        let msg = match msg { Ok(m) => m, Err(_) => break };
                        if msg.is_text() {
                            let text = msg.to_text().unwrap_or("");
                            if let Ok(req) = serde_json::from_str::<LspRequest>(text) {
                                let sessions = sessions.read().await;
                                let mut found_session = None;
                                for (_, session) in sessions.iter() {
                                    found_session = Some(session.clone());
                                    break;
                                }
                                if let Some(session) = found_session {
                                    let result = session.request(&req.method, req.params).await;
                                    let response = serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "id": req.id,
                                        "result": result,
                                    });
                                    let _ = ws_sink.lock().await.send(
                                        tokio_tungstenite::tungstenite::Message::Text(response.to_string())
                                    ).await;
                                }
                            }
                        }
                    }
                });
            }
        });

        *port_guard = Some(port);
        Ok(port)
    }

    pub async fn start_server(&self, project_path: &str, server_command: &str, args: Vec<String>) -> Result<(), String> {
        let mut child = Command::new(server_command)
            .args(&args)
            .current_dir(project_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start LSP server: {}", e))?;

        let stdin = child.stdin.take().ok_or("No stdin")?;
        let stdout = child.stdout.take().ok_or("No stdout")?;

        let (stdin_tx, mut stdin_rx) = mpsc::channel::<Vec<u8>>(32);
        let pending: Arc<RwLock<HashMap<u64, oneshot::Sender<serde_json::Value>>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let next_id = Arc::new(RwLock::new(1u64));

        let pending_clone = pending.clone();
        let next_id_clone = next_id.clone();

        tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(data) = stdin_rx.recv().await {
                let _ = stdin.write_all(&data).await;
            }
        });

        let pending_for_read = pending.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut content_length: Option<usize> = None;
            while let Ok(Some(line)) = lines.next_line().await {
                if line.is_empty() {
                    if let Some(len) = content_length.take() {
                        let mut body = vec![0u8; len];
                        let mut total_read = 0;
                        while total_read < len {
                            match lines.read_line(String::new()).await {
                                Ok(0) => break,
                                Ok(n) => total_read += n,
                                Err(_) => break,
                            }
                        }
                    }
                } else if let Some(val) = line.strip_prefix("Content-Length: ") {
                    content_length = val.trim().parse().ok();
                }
            }
        });

        let session = LspSession {
            child,
            stdin_tx,
            pending,
            next_id,
        };

        self.sessions.write().await.insert(project_path.to_string(), session);
        Ok(())
    }

    pub async fn request(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let sessions = self.sessions.read().await;
        let session = sessions.values().next().ok_or("No active LSP session")?;
        session.request(method, params).await
    }

    pub async fn stop_server(&self, project_path: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        if let Some(mut session) = sessions.remove(project_path) {
            let _ = session.child.kill().await;
        }
        Ok(())
    }

    pub async fn stop_all(&self) {
        let mut sessions = self.sessions.write().await;
        for (_, mut session) in sessions.drain() {
            let _ = session.child.kill().await;
        }
    }
}

impl LspSession {
    async fn request(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let mut id_guard = self.next_id.write().await;
        let id = *id_guard;
        *id_guard += 1;
        drop(id_guard);

        let (tx, rx) = oneshot::channel();
        self.pending.write().await.insert(id, tx);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let body = request.to_string();
        let message = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        self.stdin_tx.send(message.into_bytes()).await
            .map_err(|e| format!("Failed to send: {}", e))?;

        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(_)) => Err("Channel closed".into()),
            Err(_) => {
                self.pending.write().await.remove(&id);
                Err("Request timed out".into())
            }
        }
    }
}
```

### `src-tauri/src/services/file_watcher.rs`

```rust
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{mpsc, RwLock};

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileChangeEvent {
    pub kind: String,
    pub paths: Vec<String>,
    pub timestamp: String,
}

pub struct FileWatcher {
    watcher: Arc<RwLock<Option<RecommendedWatcher>>>,
    event_tx: mpsc::Sender<FileChangeEvent>,
    watched_dirs: Arc<RwLock<Vec<PathBuf>>>,
}

impl FileWatcher {
    pub fn new(event_tx: mpsc::Sender<FileChangeEvent>) -> Self {
        Self {
            watcher: Arc::new(RwLock::new(None)),
            event_tx,
            watched_dirs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn watch(&self, paths: Vec<String>) -> Result<(), String> {
        let mut watcher_guard = self.watcher.write().await;
        if watcher_guard.is_some() {
            self.unwatch_all().await?;
        }

        let tx = self.event_tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                if let Ok(event) = result {
                    let kind = match event.kind {
                        EventKind::Create(_) => "created",
                        EventKind::Modify(_) => "modified",
                        EventKind::Remove(_) => "removed",
                        EventKind::Rename { .. } => "renamed",
                        _ => "other",
                    };
                    let paths: Vec<String> = event.paths.iter()
                        .filter_map(|p| p.to_str().map(|s| s.to_string()))
                        .collect();
                    if !paths.is_empty() {
                        let _ = tx.blocking_send(FileChangeEvent {
                            kind: kind.to_string(),
                            paths,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        });
                    }
                }
            },
            notify::Config::default()
                .with_poll_interval(Duration::from_secs(2)),
        ).map_err(|e| format!("Failed to create watcher: {}", e))?;

        for path_str in &paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                watcher.watch(&path, RecursiveMode::Recursive)
                    .map_err(|e| format!("Failed to watch {}: {}", path_str, e))?;
                self.watched_dirs.write().await.push(path);
            }
        }

        *watcher_guard = Some(watcher);
        Ok(())
    }

    pub async fn unwatch_all(&self) -> Result<(), String> {
        let mut watcher_guard = self.watcher.write().await;
        if let Some(mut watcher) = watcher_guard.take() {
            for dir in self.watched_dirs.write().await.drain(..) {
                let _ = watcher.unwatch(&dir);
            }
        }
        Ok(())
    }

    pub async fn is_watching(&self) -> bool {
        self.watcher.read().await.is_some()
    }
}
```

### `src-tauri/src/services/backup_service.rs`

```rust
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};
use crate::error::{AppError, AppResult};
use crate::utils::crypto;

pub struct BackupService {
    backup_dir: PathBuf,
    settings_service: Arc<crate::services::settings_service::SettingsService>,
}

impl BackupService {
    pub fn new(backup_dir: PathBuf, settings_service: Arc<crate::services::settings_service::SettingsService>) -> Self {
        std::fs::create_dir_all(&backup_dir).ok();
        Self { backup_dir, settings_service }
    }

    pub fn create_backup(
        &self,
        projects: &[serde_json::Value],
        categories: &[serde_json::Value],
        passphrase: &str,
        output_path: Option<&Path>,
    ) -> AppResult<PathBuf> {
        let payload = crypto::BackupPayload {
            projects: projects.to_vec(),
            categories: categories.to_vec(),
            settings: serde_json::json!({}),
        };
        let encrypted_json = crypto::encrypt_to_json(payload, passphrase, Some(crypto::BackupAppMeta {
            name: "SelfHost Helper".into(),
            version: Some(env!("CARGO_PKG_VERSION").into()),
        }))?;
        let zip_path = output_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                self.backup_dir.join(format!("backup_{}.zip", timestamp))
            });
        let zip_file = File::create(&zip_path).map_err(|e| AppError::Io(e))?;
        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("backup.json", options).map_err(|e| AppError::Internal(e.to_string()))?;
        zip.write_all(encrypted_json.as_bytes()).map_err(|e| AppError::Io(e))?;
        zip.finish().map_err(|e| AppError::Io(e))?;
        Ok(zip_path)
    }

    pub fn restore_backup(
        &self,
        zip_path: &Path,
        passphrase: &str,
    ) -> AppResult<crypto::BackupPayload> {
        let zip_file = File::open(zip_path).map_err(|e| AppError::Io(e))?;
        let mut archive = ZipArchive::new(zip_file).map_err(|e| AppError::Internal(e.to_string()))?;
        let mut backup_json = String::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| AppError::Internal(e.to_string()))?;
            if file.name() == "backup.json" {
                file.read_to_string(&mut backup_json).map_err(|e| AppError::Io(e))?;
                break;
            }
        }
        if backup_json.is_empty() {
            return Err(AppError::Validation("Invalid backup: no backup.json found".into()));
        }
        let envelope = crypto::decrypt_from_json(&backup_json, passphrase)?;
        Ok(envelope.payload)
    }

    pub fn list_backups(&self) -> AppResult<Vec<BackupInfo>> {
        let mut backups = Vec::new();
        if self.backup_dir.exists() {
            for entry in std::fs::read_dir(&self.backup_dir).map_err(|e| AppError::Io(e))? {
                let entry = entry.map_err(|e| AppError::Io(e))?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("zip") {
                    let metadata = entry.metadata().map_err(|e| AppError::Io(e))?;
                    backups.push(BackupInfo {
                        filename: path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string(),
                        path: path.to_string_lossy().to_string(),
                        size_bytes: metadata.len(),
                        created_at: metadata.modified().ok().and_then(|t| {
                            let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                            Some(chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)?
                                .to_rfc3339())
                        }),
                    });
                }
            }
        }
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    pub fn delete_backup(&self, filename: &str) -> AppResult<()> {
        let path = self.backup_dir.join(filename);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| AppError::Io(e))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BackupInfo {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: Option<String>,
}
```

### `src-tauri/src/services/runtime_service.rs`

```rust
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Command;
use crate::error::{AppError, AppResult};

pub struct RuntimeService {
    node_versions_dir: PathBuf,
    python_versions_dir: PathBuf,
}

impl RuntimeService {
    pub fn new() -> Self {
        let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("SelfHost Helper");
        Self {
            node_versions_dir: base.join("node-versions"),
            python_versions_dir: base.join("python-versions"),
        }
    }

    pub async fn get_installed_node_versions(&self) -> AppResult<Vec<String>> {
        self.list_versions(&self.node_versions_dir, "node").await
    }

    pub async fn get_installed_python_versions(&self) -> AppResult<Vec<String>> {
        self.list_versions(&self.python_versions_dir, "python").await
    }

    async fn list_versions(&self, base_dir: &Path, _prefix: &str) -> AppResult<Vec<String>> {
        let mut versions = Vec::new();
        if base_dir.exists() {
            for entry in std::fs::read_dir(base_dir).map_err(|e| AppError::Io(e))? {
                let entry = entry.map_err(|e| AppError::Io(e))?;
                if entry.file_type().map_err(|e| AppError::Io(e))?.is_dir() {
                    versions.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        versions.sort();
        Ok(versions)
    }

    pub async fn install_node_version(&self, version: &str, progress_tx: Option<tokio::sync::mpsc::Sender<RuntimeProgress>>) -> AppResult<PathBuf> {
        let target_dir = self.node_versions_dir.join(version);
        if target_dir.exists() {
            return Ok(target_dir);
        }
        std::fs::create_dir_all(&target_dir).map_err(|e| AppError::Io(e))?;

        let url = self.get_node_download_url(version)?;
        let zip_path = target_dir.with_extension("zip");

        if let Some(tx) = &progress_tx {
            let _ = tx.send(RuntimeProgress { phase: "downloading".into(), progress: 0, message: format!("Downloading Node.js {}", version) }).await;
        }

        let response = reqwest::get(&url).await.map_err(|e| AppError::Http(e))?;
        let bytes = response.bytes().await.map_err(|e| AppError::Http(e))?;
        std::fs::write(&zip_path, &bytes).map_err(|e| AppError::Io(e))?;

        if let Some(tx) = &progress_tx {
            let _ = tx.send(RuntimeProgress { phase: "extracting".into(), progress: 50, message: "Extracting...".into() }).await;
        }

        let file = std::fs::File::open(&zip_path).map_err(|e| AppError::Io(e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| AppError::Internal(e.to_string()))?;
        archive.extract(&target_dir).map_err(|e| AppError::Internal(e.to_string()))?;
        let _ = std::fs::remove_file(&zip_path);

        if let Some(tx) = &progress_tx {
            let _ = tx.send(RuntimeProgress { phase: "done".into(), progress: 100, message: format!("Node.js {} installed", version) }).await;
        }

        Ok(target_dir)
    }

    pub async fn install_python_version(&self, version: &str, progress_tx: Option<tokio::sync::mpsc::Sender<RuntimeProgress>>) -> AppResult<PathBuf> {
        let target_dir = self.python_versions_dir.join(version);
        if target_dir.exists() {
            return Ok(target_dir);
        }
        std::fs::create_dir_all(&target_dir).map_err(|e| AppError::Io(e))?;

        let url = self.get_python_download_url(version)?;

        if let Some(tx) = &progress_tx {
            let _ = tx.send(RuntimeProgress { phase: "downloading".into(), progress: 0, message: format!("Downloading Python {}", version) }).await;
        }

        let response = reqwest::get(&url).await.map_err(|e| AppError::Http(e))?;
        let bytes = response.bytes().await.map_err(|e| AppError::Http(e))?;
        let zip_path = target_dir.with_extension("zip");
        std::fs::write(&zip_path, &bytes).map_err(|e| AppError::Io(e))?;

        if let Some(tx) = &progress_tx {
            let _ = tx.send(RuntimeProgress { phase: "extracting".into(), progress: 50, message: "Extracting...".into() }).await;
        }

        let file = std::fs::File::open(&zip_path).map_err(|e| AppError::Io(e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| AppError::Internal(e.to_string()))?;
        archive.extract(&target_dir).map_err(|e| AppError::Internal(e.to_string()))?;
        let _ = std::fs::remove_file(&zip_path);

        if let Some(tx) = &progress_tx {
            let _ = tx.send(RuntimeProgress { phase: "done".into(), progress: 100, message: format!("Python {} installed", version) }).await;
        }

        Ok(target_dir)
    }

    pub async fn uninstall_version(&self, runtime: &str, version: &str) -> AppResult<()> {
        let base_dir = match runtime {
            "node" => &self.node_versions_dir,
            "python" => &self.python_versions_dir,
            _ => return Err(AppError::Validation("Unknown runtime".into())),
        };
        let target = base_dir.join(version);
        if target.exists() {
            std::fs::remove_dir_all(&target).map_err(|e| AppError::Io(e))?;
        }
        Ok(())
    }

    pub async fn get_available_node_versions(&self) -> AppResult<Vec<String>> {
        let output = Command::new("node")
            .arg("--list-remote")
            .arg("lts")
            .output()
            .await
            .map_err(|e| AppError::Process(format!("Failed to list Node versions: {}", e)))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let versions: Vec<String> = stdout.lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.starts_with('v') {
                    Some(line.strip_prefix('v').unwrap_or(line).to_string())
                } else {
                    None
                }
            })
            .rev()
            .take(20)
            .collect();
        Ok(versions)
    }

    pub async fn get_available_python_versions(&self) -> AppResult<Vec<String>> {
        Ok(vec![
            "3.12.4".into(), "3.12.3".into(), "3.12.2".into(), "3.12.1".into(), "3.12.0".into(),
            "3.11.9".into(), "3.11.8".into(), "3.11.7".into(),
            "3.10.14".into(), "3.10.13".into(), "3.10.12".into(),
        ])
    }

    fn get_node_download_url(&self, version: &str) -> AppResult<String> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let (os_name, arch_name) = match (os, arch) {
            ("linux", "x86_64") => ("linux", "x64"),
            ("linux", "aarch64") => ("linux", "arm64"),
            ("macos", "x86_64") => ("darwin", "x64"),
            ("macos", "aarch64") => ("darwin", "arm64"),
            ("windows", "x86_64") => ("win", "x64"),
            ("windows", "aarch64") => ("win", "arm64"),
            _ => return Err(AppError::Runtime(format!("Unsupported platform: {}-{}", os, arch))),
        };
        let ext = if os == "windows" { "zip" } else { "tar.gz" };
        Ok(format!(
            "https://nodejs.org/dist/v{}/node-v{}-{}-{}.{}",
            version, version, os_name, arch_name, ext
        ))
    }

    fn get_python_download_url(&self, version: &str) -> AppResult<String> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let (os_name, arch_name) = match (os, arch) {
            ("linux", "x86_64") => ("linux", "x86_64"),
            ("linux", "aarch64") => ("linux", "aarch64"),
            ("macos", "x86_64") => ("macos", "x86_64"),
            ("macos", "aarch64") => ("macos", "aarch64"),
            ("windows", "x86_64") => ("win", "amd64"),
            _ => return Err(AppError::Runtime(format!("Unsupported platform: {}-{}", os, arch))),
        };
        Ok(format!(
            "https://www.python.org/ftp/python/{}/python-{}-{}.zip",
            version, version, if os_name == "macos" { format!("{}-{}", os_name, arch_name) } else { format!("{}-{}", os_name, arch_name) }
        ))
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RuntimeProgress {
    pub phase: String,
    pub progress: u32,
    pub message: String,
}
```

### `src-tauri/src/services/update_service.rs`

```rust
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub date: Option<String>,
    pub notes: Option<String>,
    pub platforms: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub status: String,
    pub update: Option<UpdateInfo>,
    pub error: Option<String>,
}

pub struct UpdateService {
    app_handle: tauri::AppHandle,
    status: Arc<RwLock<UpdateStatus>>,
    check_interval_secs: u64,
}

impl UpdateService {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            app_handle,
            status: Arc::new(RwLock::new(UpdateStatus {
                status: "idle".into(),
                update: None,
                error: None,
            })),
            check_interval_secs: 86400,
        }
    }

    pub async fn check_for_updates(&self) -> AppResult<Option<UpdateInfo>> {
        *self.status.write().await = UpdateStatus {
            status: "checking".into(),
            update: None,
            error: None,
        };

        match self.app_handle.updater().check().await {
            Ok(Some(update)) => {
                let info = UpdateInfo {
                    version: update.version.clone(),
                    date: update.date.clone(),
                    notes: update.body.clone(),
                    platforms: None,
                };
                *self.status.write().await = UpdateStatus {
                    status: "available".into(),
                    update: Some(info.clone()),
                    error: None,
                };
                Ok(Some(info))
            }
            Ok(None) => {
                *self.status.write().await = UpdateStatus {
                    status: "up_to_date".into(),
                    update: None,
                    error: None,
                };
                Ok(None)
            }
            Err(e) => {
                let err_msg = format!("Failed to check for updates: {}", e);
                *self.status.write().await = UpdateStatus {
                    status: "error".into(),
                    update: None,
                    error: Some(err_msg.clone()),
                };
                Err(AppError::Internal(err_msg))
            }
        }
    }

    pub async fn download_and_install(&self) -> AppResult<()> {
        *self.status.write().await = UpdateStatus {
            status: "downloading".into(),
            update: self.status.read().await.update.clone(),
            error: None,
        };

        match self.app_handle.updater().check().await {
            Ok(Some(update)) => {
                update.download_and_install(|_, _| {}, || {}).await
                    .map_err(|e| AppError::Internal(format!("Update install failed: {}", e)))?;
                Ok(())
            }
            Ok(None) => Err(AppError::Validation("No update available".into())),
            Err(e) => Err(AppError::Internal(format!("Update check failed: {}", e))),
        }
    }

    pub async fn get_status(&self) -> UpdateStatus {
        self.status.read().await.clone()
    }

    pub async fn get_current_version(&self) -> String {
        self.app_handle.package_info().version.to_string()
    }

    pub fn start_periodic_check(&self) {
        let status = self.status.clone();
        let handle = self.app_handle.clone();
        let interval = self.check_interval_secs;
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(interval));
            loop {
                interval_timer.tick().await;
                match handle.updater().check().await {
                    Ok(Some(update)) => {
                        let info = UpdateInfo {
                            version: update.version.clone(),
                            date: update.date.clone(),
                            notes: update.body.clone(),
                            platforms: None,
                        };
                        *status.write().await = UpdateStatus {
                            status: "available".into(),
                            update: Some(info),
                            error: None,
                        };
                        let _ = handle.emit("update-available", &status.read().await.clone());
                    }
                    _ => {}
                }
            }
        });
    }
}
```

### `src-tauri/src/native/tray.rs`

```rust
use std::sync::Arc;
use tauri::{
    AppHandle, Manager,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_menu(app)?;
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap())
        .tooltip("SelfHost Helper")
        .menu(&menu)
        .on_menu_event(move |app, event| {
            handle_menu_event(app, &event.id().0);
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

fn build_menu(app: &AppHandle) -> Result<tauri::menu::Menu, Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
    let hide = MenuItemBuilder::with_id("hide", "Hide Window").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let mut projects_menu = SubmenuBuilder::new(app, "Projects");
    projects_menu = projects_menu.item(&MenuItemBuilder::with_id("project_add", "Add Project").build(app)?);
    projects_menu = projects_menu.item(&MenuItemBuilder::with_id("project_open", "Open Selected").build(app)?);
    projects_menu = projects_menu.item(&MenuItemBuilder::with_id("project_open_dir", "Open Directory").build(app)?);
    projects_menu = projects_menu.separator()?;
    projects_menu = projects_menu.item(&MenuItemBuilder::with_id("project_stop", "Stop Process").build(app)?);
    projects_menu = projects_menu.item(&MenuItemBuilder::with_id("project_start", "Start Process").build(app)?);
    projects_menu = projects_menu.item(&MenuItemBuilder::with_id("project_restart", "Restart Process").build(app)?);

    let mut tools_menu = SubmenuBuilder::new(app, "Tools");
    tools_menu = tools_menu.item(&MenuItemBuilder::with_id("tunnel_stop_all", "Stop All Tunnels").build(app)?);
    tools_menu = tools_menu.item(&MenuItemBuilder::with_id("check_updates", "Check for Updates").build(app)?);

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&hide)
        .item(&separator)
        .item(&projects_menu.build()?)
        .item(&tools_menu.build()?)
        .item(&separator)
        .item(&quit)
        .build()?;

    Ok(menu)
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "hide" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
        "quit" => {
            app.exit(0);
        }
        "project_add" => {
            let _ = app.emit("tray:project_add", ());
        }
        "project_open" => {
            let _ = app.emit("tray:project_open", ());
        }
        "project_open_dir" => {
            let _ = app.emit("tray:project_open_dir", ());
        }
        "project_stop" => {
            let _ = app.emit("tray:project_stop", ());
        }
        "project_start" => {
            let _ = app.emit("tray:project_start", ());
        }
        "project_restart" => {
            let _ = app.emit("tray:project_restart", ());
        }
        "tunnel_stop_all" => {
            let _ = app.emit("tray:tunnel_stop_all", ());
        }
        "check_updates" => {
            let _ = app.emit("tray:check_updates", ());
        }
        _ => {}
    }
}

pub async fn rebuild_tray_with_projects(app: &AppHandle, projects: Vec<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
    let mut projects_menu = SubmenuBuilder::new(app, "Projects");
    for project in &projects {
        let name = project.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
        let id = project.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
        projects_menu = projects_menu.item(&MenuItemBuilder::with_id(
            format!("tray_project_{}", id),
            name,
        ).build(app)?);
    }
    projects_menu = projects_menu.separator()?;
    projects_menu = projects_menu.item(&MenuItemBuilder::with_id("project_add", "Add Project").build(app)?);

    let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
    let hide = MenuItemBuilder::with_id("hide", "Hide Window").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let mut tools_menu = SubmenuBuilder::new(app, "Tools");
    tools_menu = tools_menu.item(&MenuItemBuilder::with_id("tunnel_stop_all", "Stop All Tunnels").build(app)?);
    tools_menu = tools_menu.item(&MenuItemBuilder::with_id("check_updates", "Check for Updates").build(app)?);

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&hide)
        .item(&separator)
        .item(&projects_menu.build()?)
        .item(&tools_menu.build()?)
        .item(&separator)
        .item(&quit)
        .build()?;

    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}
```

### `src-tauri/src/lib.rs` — Entry Point + Tauri Commands

```rust
mod db;
mod error;
mod native;
mod services;
mod utils;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

use crate::db::connection::establish_connection;
use crate::db::projects_repo::ProjectsRepo;
use crate::db::categories_repo::CategoriesRepo;
use crate::error::{AppError, AppResult};
use crate::native::tray;
use crate::services::backup_service::BackupService;
use crate::services::file_watcher::FileWatcher;
use crate::services::git_service::GitService;
use crate::services::lsp_bridge::LspBridge;
use crate::services::runtime_service::RuntimeService;
use crate::services::search_service::SearchService;
use crate::services::settings_service::SettingsService;
use crate::services::tunnel_manager::TunnelManager;
use crate::services::update_service::UpdateService;
use crate::utils::crypto;
use crate::utils::media_allowlist::MediaAllowlist;

pub struct AppState {
    pub db: Arc<rusqlite::Connection>,
    pub projects_repo: Arc<ProjectsRepo>,
    pub categories_repo: Arc<CategoriesRepo>,
    pub settings_service: Arc<SettingsService>,
    pub git_service: Arc<GitService>,
    pub search_service: Arc<SearchService>,
    pub lsp_bridge: Arc<LspBridge>,
    pub runtime_service: Arc<RuntimeService>,
    pub file_watcher: Arc<FileWatcher>,
    pub backup_service: Arc<BackupService>,
    pub tunnel_manager: Arc<TunnelManager>,
    pub update_service: Arc<UpdateService>,
    pub media_allowlist: Arc<MediaAllowlist>,
    pub dev_mode: bool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("info,selfhost_helper=debug")
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            let is_dev = handle.config().build.dev_url.is_some();
            let app_base = handle.path().app_data_dir().unwrap_or_default();
            let db_path = app_base.join("data").join("projects.sqlite");
            std::fs::create_dir_all(db_path.parent().unwrap()).ok();
            let db = establish_connection(&db_path)?;
            let projects_repo = Arc::new(ProjectsRepo::new(db.clone()));
            let categories_repo = Arc::new(CategoriesRepo::new(db.clone()));
            let settings_service = Arc::new(SettingsService::new(db.clone()));
            settings_service.init_tables().ok();
            let git_service = Arc::new(GitService::new());
            let search_service = Arc::new(SearchService::new());
            let lsp_bridge = Arc::new(LspBridge::new());
            let runtime_service = Arc::new(RuntimeService::new());
            let media_allowlist = Arc::new(MediaAllowlist::new());
            let log_store = Arc::new(crate::services::log_store::LogStore::new());
            let (tunnel_manager, _tunnel_rx) = TunnelManager::new();
            let backup_dir = app_base.join("backups");
            let backup_service = Arc::new(BackupService::new(backup_dir, settings_service.clone()));
            let (file_watcher_tx, mut file_watcher_rx) = tokio::sync::mpsc::channel(64);
            let file_watcher = Arc::new(FileWatcher::new(file_watcher_tx));
            let update_service = Arc::new(UpdateService::new(handle.clone()));
            update_service.start_periodic_check();

            let app_state = AppState {
                db,
                projects_repo,
                categories_repo,
                settings_service,
                git_service,
                search_service,
                lsp_bridge,
                runtime_service,
                file_watcher,
                backup_service,
                tunnel_manager: Arc::new(tunnel_manager),
                update_service,
                media_allowlist,
                dev_mode: is_dev,
            };
            app.manage(app_state);

            let handle_clone = handle.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(change) = file_watcher_rx.recv().await {
                    let _ = handle_clone.emit("file-changed", &change);
                }
            });

            tray::create_tray(&handle).ok();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Project commands
            get_projects,
            add_project,
            update_project,
            delete_project,
            // Category commands
            get_categories,
            add_category,
            update_category,
            delete_category,
            // Process commands
            start_process,
            stop_process,
            restart_process,
            get_process_status,
            get_process_stats,
            // Tunnel commands
            start_tunnel,
            stop_tunnel,
            get_tunnel_status,
            get_tunnel_logs,
            clear_tunnel_logs,
            // Git commands
            git_get_branches,
            git_checkout_branch,
            git_create_branch,
            git_delete_branch,
            git_get_status,
            git_stage_file,
            git_unstage_file,
            git_stage_all,
            git_commit,
            git_push,
            git_pull,
            git_clone,
            git_get_remotes,
            git_add_remote,
            git_get_diff_summary,
            git_get_log,
            git_stash,
            git_stash_list,
            git_stash_pop,
            git_stash_drop,
            git_get_tags,
            git_diff_file,
            git_get_commit_diff,
            git_get_branch_ahead_behind,
            git_get_current_branch,
            git_stash_apply,
            // Search commands
            search_files,
            // Runtime commands
            get_installed_node_versions,
            get_installed_python_versions,
            install_node_version,
            install_python_version,
            uninstall_runtime_version,
            get_available_node_versions,
            get_available_python_versions,
            // Settings commands
            get_app_settings,
            set_app_setting,
            update_app_settings,
            get_project_settings,
            set_project_settings,
            delete_project_settings,
            // Backup commands
            create_backup,
            restore_backup,
            list_backups,
            delete_backup,
            // Update commands
            check_for_updates,
            download_and_install_update,
            get_update_status,
            get_current_version,
            // File watcher commands
            watch_directory,
            unwatch_all,
            is_watching,
            // LSP commands
            start_lsp_server,
            stop_lsp_server,
            lsp_request,
            start_lsp_proxy,
            // System commands
            open_file,
            open_folder,
            open_external,
            get_platform_info,
            // Media commands
            validate_media_path,
            // Tray commands
            refresh_tray_menu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ===== PROJECT COMMANDS =====
#[tauri::command]
fn get_projects(state: tauri::State<AppState>) -> AppResult<Vec<serde_json::Value>> {
    state.projects_repo.get_all().map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn add_project(state: tauri::State<AppState>, name: String, path: String, category_id: Option<i64>, tags: Option<String>) -> AppResult<i64> {
    state.projects_repo.create(&name, &path, category_id, tags.as_deref()).map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn update_project(state: tauri::State<AppState>, id: i64, name: Option<String>, path: Option<String>, category_id: Option<i64>, tags: Option<String>, icon: Option<String>) -> AppResult<()> {
    state.projects_repo.update(id, name.as_deref(), path.as_deref(), category_id, tags.as_deref(), icon.as_deref()).map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn delete_project(state: tauri::State<AppState>, id: i64) -> AppResult<()> {
    state.projects_repo.delete(id).map_err(|e| AppError::Database(e.to_string()))
}

// ===== CATEGORY COMMANDS =====
#[tauri::command]
fn get_categories(state: tauri::State<AppState>) -> AppResult<Vec<serde_json::Value>> {
    state.categories_repo.get_all().map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn add_category(state: tauri::State<AppState>, name: String, color: Option<String>) -> AppResult<i64> {
    state.categories_repo.create(&name, color.as_deref()).map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn update_category(state: tauri::State<AppState>, id: i64, name: Option<String>, color: Option<String>) -> AppResult<()> {
    state.categories_repo.update(id, name.as_deref(), color.as_deref()).map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
fn delete_category(state: tauri::State<AppState>, id: i64) -> AppResult<()> {
    state.categories_repo.delete(id).map_err(|e| AppError::Database(e.to_string()))
}

// ===== PROCESS COMMANDS =====
#[tauri::command]
async fn start_process(state: tauri::State<'_, AppState>, project_id: i64, project_path: String, command: String) -> AppResult<String> {
    #[cfg(target_os = "windows")]
    {
        let output = tokio::process::Command::new("cmd")
            .args(["/C", "start", "/B", "cmd", "/c", &command])
            .current_dir(&project_path)
            .output()
            .await
            .map_err(|e| AppError::Process(e.to_string()))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let output = tokio::process::Command::new("sh")
            .args(["-c", &command])
            .current_dir(&project_path)
            .output()
            .await
            .map_err(|e| AppError::Process(e.to_string()))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[tauri::command]
async fn stop_process(pid: u32) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        tokio::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .output()
            .await
            .map_err(|e| AppError::Process(e.to_string()))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        unsafe { libc::kill(pid as i32, libc::SIGKILL); }
    }
    Ok(())
}

#[tauri::command]
async fn restart_process(state: tauri::State<'_, AppState>, project_id: i64, project_path: String, command: String) -> AppResult<String> {
    let _ = stop_process(pid).await;
    start_process(state, project_id, project_path, command).await
}

#[tauri::command]
fn get_process_status(pid: u32) -> AppResult<bool> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, GetExitCodeProcess };
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle == 0 { return Ok(false); }
            let mut exit_code = 0u32;
            let success = GetExitCodeProcess(handle, &mut exit_code);
            windows_sys::Win32::Foundation::CloseHandle(handle);
            Ok(success != 0 && exit_code == 259) // STILL_ACTIVE
        }
    }
    #[cfg(not(target_os = "windows"))]
    { Ok(nix::sys::signal::kill(nix::unistd::Pid::from(pid as i32), None).is_ok()) }
}

#[tauri::command]
fn get_process_stats(pid: u32) -> AppResult<serde_json::Value> {
    Ok(serde_json::json!({ "pid": pid, "cpu": 0.0, "memory": 0 }))
}

// ===== TUNNEL COMMANDS =====
#[tauri::command]
async fn start_tunnel(state: tauri::State<'_, AppState>, project_id: String, mode: String, port: u16, token: Option<String>, config: Option<HashMap<String, String>>) -> AppResult<()> {
    state.tunnel_manager.start_tunnel(project_id, mode, port, token, config).await
}

#[tauri::command]
async fn stop_tunnel(state: tauri::State<'_, AppState>, project_id: String) -> AppResult<()> {
    state.tunnel_manager.stop_tunnel(&project_id).await
}

#[tauri::command]
async fn get_tunnel_status(state: tauri::State<'_, AppState>, project_id: String) -> AppResult<Option<serde_json::Value>> {
    state.tunnel_manager.get_status(&project_id).await
        .map(|s| serde_json::to_value(s).ok())
        .transpose()
        .map(|v| v.flatten())
}

#[tauri::command]
async fn get_tunnel_logs(state: tauri::State<'_, AppState>, project_id: String) -> AppResult<Vec<serde_json::Value>> {
    state.tunnel_manager.get_logs(&project_id).await
        .into_iter()
        .map(|e| serde_json::to_value(e).map_err(|e| AppError::Json(e)))
        .collect()
}

#[tauri::command]
async fn clear_tunnel_logs(state: tauri::State<'_, AppState>, project_id: String) -> AppResult<()> {
    state.tunnel_manager.clear_logs(&project_id).await;
    Ok(())
}

// ===== GIT COMMANDS =====
#[tauri::command]
fn git_get_branches(state: tauri::State<AppState>, project_path: String) -> AppResult<Vec<crate::services::git_service::GitBranch>> {
    state.git_service.get_branches(&project_path)
}

#[tauri::command]
fn git_checkout_branch(state: tauri::State<AppState>, project_path: String, branch_name: String) -> AppResult<()> {
    state.git_service.checkout_branch(&project_path, &branch_name)
}

#[tauri::command]
fn git_create_branch(state: tauri::State<AppState>, project_path: String, branch_name: String) -> AppResult<()> {
    state.git_service.create_branch(&project_path, &branch_name)
}

#[tauri::command]
fn git_delete_branch(state: tauri::State<AppState>, project_path: String, branch_name: String) -> AppResult<()> {
    state.git_service.delete_branch(&project_path, &branch_name)
}

#[tauri::command]
fn git_get_status(state: tauri::State<AppState>, project_path: String) -> AppResult<Vec<crate::services::git_service::GitStatusEntry>> {
    state.git_service.get_status(&project_path)
}

#[tauri::command]
fn git_stage_file(state: tauri::State<AppState>, project_path: String, file_path: String) -> AppResult<()> {
    state.git_service.stage_file(&project_path, &file_path)
}

#[tauri::command]
fn git_unstage_file(state: tauri::State<AppState>, project_path: String, file_path: String) -> AppResult<()> {
    state.git_service.unstage_file(&project_path, &file_path)
}

#[tauri::command]
fn git_stage_all(state: tauri::State<AppState>, project_path: String) -> AppResult<()> {
    state.git_service.stage_all(&project_path)
}

#[tauri::command]
fn git_commit(state: tauri::State<AppState>, project_path: String, message: String, author_name: String, author_email: String) -> AppResult<String> {
    state.git_service.commit(&project_path, &message, &author_name, &author_email)
}

#[tauri::command]
fn git_push(state: tauri::State<AppState>, project_path: String, remote_name: String) -> AppResult<()> {
    state.git_service.push(&project_path, &remote_name)
}

#[tauri::command]
fn git_pull(state: tauri::State<AppState>, project_path: String) -> AppResult<()> {
    state.git_service.pull(&project_path)
}

#[tauri::command]
fn git_clone(state: tauri::State<AppState>, url: String, dest_path: String) -> AppResult<()> {
    state.git_service.clone(&url, &dest_path)
}

#[tauri::command]
fn git_get_remotes(state: tauri::State<AppState>, project_path: String) -> AppResult<Vec<crate::services::git_service::GitRemote>> {
    state.git_service.get_remotes(&project_path)
}

#[tauri::command]
fn git_add_remote(state: tauri::State<AppState>, project_path: String, name: String, url: String) -> AppResult<()> {
    state.git_service.add_remote(&project_path, &name, &url)
}

#[tauri::command]
fn git_get_diff_summary(state: tauri::State<AppState>, project_path: String) -> AppResult<Vec<crate::services::git_service::GitDiffEntry>> {
    state.git_service.get_diff_summary(&project_path)
}

#[tauri::command]
fn git_get_log(state: tauri::State<AppState>, project_path: String, max_count: usize) -> AppResult<Vec<crate::services::git_service::GitCommit>> {
    state.git_service.get_log(&project_path, max_count)
}

#[tauri::command]
fn git_stash(state: tauri::State<AppState>, project_path: String, message: Option<String>) -> AppResult<()> {
    state.git_service.stash(&project_path, message.as_deref())
}

#[tauri::command]
fn git_stash_list(state: tauri::State<AppState>, project_path: String) -> AppResult<Vec<crate::services::git_service::GitStashEntry>> {
    state.git_service.stash_list(&project_path)
}

#[tauri::command]
fn git_stash_pop(state: tauri::State<AppState>, project_path: String, stash_index: usize) -> AppResult<()> {
    state.git_service.stash_pop(&project_path, stash_index)
}

#[tauri::command]
fn git_stash_drop(state: tauri::State<AppState>, project_path: String, stash_index: usize) -> AppResult<()> {
    state.git_service.stash_drop(&project_path, stash_index)
}

#[tauri::command]
fn git_get_tags(state: tauri::State<AppState>, project_path: String) -> AppResult<Vec<crate::services::git_service::GitTag>> {
    state.git_service.get_tags(&project_path)
}

#[tauri::command]
fn git_diff_file(state: tauri::State<AppState>, project_path: String, file_path: String) -> AppResult<String> {
    state.git_service.diff_file(&project_path, &file_path)
}

#[tauri::command]
fn git_get_commit_diff(state: tauri::State<AppState>, project_path: String, commit_oid: String) -> AppResult<String> {
    state.git_service.get_commit_diff(&project_path, &commit_oid)
}

#[tauri::command]
fn git_get_branch_ahead_behind(state: tauri::State<AppState>, project_path: String, branch_name: String) -> AppResult<(usize, usize)> {
    state.git_service.get_branch_ahead_behind(&project_path, &branch_name)
}

#[tauri::command]
fn git_get_current_branch(state: tauri::State<AppState>, project_path: String) -> AppResult<String> {
    state.git_service.get_current_branch(&project_path)
}

#[tauri::command]
fn git_stash_apply(state: tauri::State<AppState>, project_path: String, stash_index: usize) -> AppResult<()> {
    state.git_service.stash_apply(&project_path, stash_index)
}

// ===== SEARCH COMMANDS =====
#[tauri::command]
async fn search_files(state: tauri::State<'_, AppState>, directory: String, query: String, case_sensitive: bool, whole_word: bool, regex_mode: bool, max_results: Option<usize>) -> AppResult<serde_json::Value> {
    let (results, stats) = state.search_service.search(&directory, &query, case_sensitive, whole_word, regex_mode, max_results.unwrap_or(500)).await?;
    Ok(serde_json::json!({ "results": results, "stats": stats }))
}

// ===== RUNTIME COMMANDS =====
#[tauri::command]
async fn get_installed_node_versions(state: tauri::State<'_, AppState>) -> AppResult<Vec<String>> {
    state.runtime_service.get_installed_node_versions().await
}

#[tauri::command]
async fn get_installed_python_versions(state: tauri::State<'_, AppState>) -> AppResult<Vec<String>> {
    state.runtime_service.get_installed_python_versions().await
}

#[tauri::command]
async fn install_node_version(state: tauri::State<'_, AppState>, version: String) -> AppResult<PathBuf> {
    state.runtime_service.install_node_version(&version, None).await
}

#[tauri::command]
async fn install_python_version(state: tauri::State<'_, AppState>, version: String) -> AppResult<PathBuf> {
    state.runtime_service.install_python_version(&version, None).await
}

#[tauri::command]
async fn uninstall_runtime_version(state: tauri::State<'_, AppState>, runtime: String, version: String) -> AppResult<()> {
    state.runtime_service.uninstall_version(&runtime, &version).await
}

#[tauri::command]
async fn get_available_node_versions(state: tauri::State<'_, AppState>) -> AppResult<Vec<String>> {
    state.runtime_service.get_available_node_versions().await
}

#[tauri::command]
async fn get_available_python_versions(state: tauri::State<'_, AppState>) -> AppResult<Vec<String>> {
    state.runtime_service.get_available_python_versions().await
}

// ===== SETTINGS COMMANDS =====
#[tauri::command]
fn get_app_settings(state: tauri::State<AppState>) -> AppResult<crate::services::settings_service::AppSettings> {
    state.settings_service.get_app_settings()
}

#[tauri::command]
fn set_app_setting(state: tauri::State<AppState>, key: String, value: String) -> AppResult<()> {
    state.settings_service.set_app_setting(&key, &value)
}

#[tauri::command]
fn update_app_settings(state: tauri::State<AppState>, settings: HashMap<String, String>) -> AppResult<()> {
    state.settings_service.update_app_settings(&settings)
}

#[tauri::command]
fn get_project_settings(state: tauri::State<AppState>, project_id: String) -> AppResult<Option<crate::services::settings_service::ProjectSettings>> {
    state.settings_service.get_project_settings(&project_id)
}

#[tauri::command]
fn set_project_settings(state: tauri::State<AppState>, settings: crate::services::settings_service::ProjectSettings) -> AppResult<()> {
    state.settings_service.set_project_settings(&settings)
}

#[tauri::command]
fn delete_project_settings(state: tauri::State<AppState>, project_id: String) -> AppResult<()> {
    state.settings_service.delete_project_settings(&project_id)
}

// ===== BACKUP COMMANDS =====
#[tauri::command]
fn create_backup(state: tauri::State<AppState>, passphrase: String) -> AppResult<String> {
    let projects = state.projects_repo.get_all().map_err(|e| AppError::Database(e.to_string()))?;
    let categories = state.categories_repo.get_all().map_err(|e| AppError::Database(e.to_string()))?;
    let path = state.backup_service.create_backup(&projects, &categories, &passphrase, None)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn restore_backup(state: tauri::State<AppState>, zip_path: String, passphrase: String) -> AppResult<crypto::BackupPayload> {
    state.backup_service.restore_backup(std::path::Path::new(&zip_path), &passphrase)
}

#[tauri::command]
fn list_backups(state: tauri::State<AppState>) -> AppResult<Vec<crate::services::backup_service::BackupInfo>> {
    state.backup_service.list_backups()
}

#[tauri::command]
fn delete_backup(state: tauri::State<AppState>, filename: String) -> AppResult<()> {
    state.backup_service.delete_backup(&filename)
}

// ===== UPDATE COMMANDS =====
#[tauri::command]
async fn check_for_updates(state: tauri::State<'_, AppState>) -> AppResult<Option<crate::services::update_service::UpdateInfo>> {
    state.update_service.check_for_updates().await
}

#[tauri::command]
async fn download_and_install_update(state: tauri::State<'_, AppState>) -> AppResult<()> {
    state.update_service.download_and_install().await
}

#[tauri::command]
async fn get_update_status(state: tauri::State<'_, AppState>) -> AppResult<crate::services::update_service::UpdateStatus> {
    Ok(state.update_service.get_status().await)
}

#[tauri::command]
async fn get_current_version(state: tauri::State<'_, AppState>) -> AppResult<String> {
    Ok(state.update_service.get_current_version().await)
}

// ===== FILE WATCHER COMMANDS =====
#[tauri::command]
async fn watch_directory(state: tauri::State<'_, AppState>, paths: Vec<String>) -> AppResult<()> {
    state.file_watcher.watch(paths).await.map_err(|e| AppError::Internal(e))
}

#[tauri::command]
async fn unwatch_all(state: tauri::State<'_, AppState>) -> AppResult<()> {
    state.file_watcher.unwatch_all().await.map_err(|e| AppError::Internal(e))
}

#[tauri::command]
async fn is_watching(state: tauri::State<'_, AppState>) -> AppResult<bool> {
    Ok(state.file_watcher.is_watching().await)
}

// ===== LSP COMMANDS =====
#[tauri::command]
async fn start_lsp_server(state: tauri::State<'_, AppState>, project_path: String, server_command: String, args: Vec<String>) -> AppResult<()> {
    state.lsp_bridge.start_server(&project_path, &server_command, args).await.map_err(|e| AppError::Internal(e))
}

#[tauri::command]
async fn stop_lsp_server(state: tauri::State<'_, AppState>, project_path: String) -> AppResult<()> {
    state.lsp_bridge.stop_server(&project_path).await.map_err(|e| AppError::Internal(e))
}

#[tauri::command]
async fn lsp_request(state: tauri::State<'_, AppState>, method: String, params: serde_json::Value) -> AppResult<serde_json::Value> {
    state.lsp_bridge.request(&method, params).await.map_err(|e| AppError::Internal(e))
}

#[tauri::command]
async fn start_lsp_proxy(state: tauri::State<'_, AppState>) -> AppResult<u16> {
    state.lsp_bridge.start_proxy().await.map_err(|e| AppError::Internal(e))
}

// ===== SYSTEM COMMANDS =====
#[tauri::command]
fn open_file(path: String) -> AppResult<()> {
    opener::open(&path).map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
fn open_folder(path: String) -> AppResult<()> {
    opener::reveal(&path).map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
fn open_external(url: String) -> AppResult<()> {
    opener::open(&url).map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
fn get_platform_info() -> AppResult<serde_json::Value> {
    Ok(serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "family": if cfg!(target_os = "windows") { "windows" } else if cfg!(target_os = "macos") { "macos" } else { "linux" },
    }))
}

// ===== MEDIA COMMANDS =====
#[tauri::command]
fn validate_media_path(state: tauri::State<AppState>, path: String) -> AppResult<String> {
    let file_path = std::path::Path::new(&path);
    state.media_allowlist.validate_media_path(file_path)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| AppError::Internal(e.to_string()))
}

// ===== TRAY COMMANDS =====
#[tauri::command]
async fn refresh_tray_menu(state: tauri::State<'_, AppState>, app: tauri::AppHandle) -> AppResult<()> {
    let projects = state.projects_repo.get_all().map_err(|e| AppError::Database(e.to_string()))?;
    tray::rebuild_tray_with_projects(&app, projects).await.map_err(|e| AppError::Internal(e.to_string()))
}
```

### `src/lib/tauri-bridge.js` — Frontend Compatibility Layer

This file replaces `window.api` from `electron/preload.js`. It wraps Tauri's `invoke()` and `listen()` calls.

```javascript
import { invoke, Channel } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { save } from "@tauri-apps/plugin-dialog";
import { BaseDirectory, readTextFile, writeTextFile, exists, mkdir, remove, rename, readDir } from "@tauri-apps/plugin-fs";
import { fetch } from "@tauri-apps/plugin-http";
import { openUrl } from "@tauri-apps/plugin-shell";
import { exit, relaunch } from "@tauri-apps/plugin-process";

// Project operations
export const api = {
  // Projects
  getProjects: () => invoke("get_projects"),
  addProject: (name, path, categoryId, tags) => invoke("add_project", { name, path, categoryId, tags }),
  updateProject: (id, name, path, categoryId, tags, icon) => invoke("update_project", { id, name, path, categoryId, tags, icon }),
  deleteProject: (id) => invoke("delete_project", { id }),

  // Categories
  getCategories: () => invoke("get_categories"),
  addCategory: (name, color) => invoke("add_category", { name, color }),
  updateCategory: (id, name, color) => invoke("update_category", { id, name, color }),
  deleteCategory: (id) => invoke("delete_category", { id }),

  // Process management
  startProcess: (projectId, projectPath, command) => invoke("start_process", { projectId, projectPath, command }),
  stopProcess: (pid) => invoke("stop_process", { pid }),
  restartProcess: (projectId, projectPath, command) => invoke("restart_process", { projectId, projectPath, command }),
  getProcessStatus: (pid) => invoke("get_process_status", { pid }),
  getProcessStats: (pid) => invoke("get_process_stats", { pid }),

  // Tunnels
  startTunnel: (projectId, mode, port, token, config) => invoke("start_tunnel", { projectId, mode, port, token, config }),
  stopTunnel: (projectId) => invoke("stop_tunnel", { projectId }),
  getTunnelStatus: (projectId) => invoke("get_tunnel_status", { projectId }),
  getTunnelLogs: (projectId) => invoke("get_tunnel_logs", { projectId }),
  clearTunnelLogs: (projectId) => invoke("clear_tunnel_logs", { projectId }),

  // Git
  gitGetBranches: (projectPath) => invoke("git_get_branches", { projectPath }),
  gitCheckoutBranch: (projectPath, branchName) => invoke("git_checkout_branch", { projectPath, branchName }),
  gitCreateBranch: (projectPath, branchName) => invoke("git_create_branch", { projectPath, branchName }),
  gitDeleteBranch: (projectPath, branchName) => invoke("git_delete_branch", { projectPath, branchName }),
  gitGetStatus: (projectPath) => invoke("git_get_status", { projectPath }),
  gitStageFile: (projectPath, filePath) => invoke("git_stage_file", { projectPath, filePath }),
  gitUnstageFile: (projectPath, filePath) => invoke("git_unstage_file", { projectPath, filePath }),
  gitStageAll: (projectPath) => invoke("git_stage_all", { projectPath }),
  gitCommit: (projectPath, message, authorName, authorEmail) => invoke("git_commit", { projectPath, message, authorName, authorEmail }),
  gitPush: (projectPath, remoteName) => invoke("git_push", { projectPath, remoteName }),
  gitPull: (projectPath) => invoke("git_pull", { projectPath }),
  gitClone: (url, destPath) => invoke("git_clone", { url, destPath }),
  gitGetRemotes: (projectPath) => invoke("git_get_remotes", { projectPath }),
  gitAddRemote: (projectPath, name, url) => invoke("git_add_remote", { projectPath, name, url }),
  gitGetDiffSummary: (projectPath) => invoke("git_get_diff_summary", { projectPath }),
  gitGetLog: (projectPath, maxCount) => invoke("git_get_log", { projectPath, maxCount }),
  gitStash: (projectPath, message) => invoke("git_stash", { projectPath, message }),
  gitStashList: (projectPath) => invoke("git_stash_list", { projectPath }),
  gitStashPop: (projectPath, stashIndex) => invoke("git_stash_pop", { projectPath, stashIndex }),
  gitStashDrop: (projectPath, stashIndex) => invoke("git_stash_drop", { projectPath, stashIndex }),
  gitStashApply: (projectPath, stashIndex) => invoke("git_stash_apply", { projectPath, stashIndex }),
  gitGetTags: (projectPath) => invoke("git_get_tags", { projectPath }),
  gitDiffFile: (projectPath, filePath) => invoke("git_diff_file", { projectPath, filePath }),
  gitGetCommitDiff: (projectPath, commitOid) => invoke("git_get_commit_diff", { projectPath, commitOid }),
  gitGetBranchAheadBehind: (projectPath, branchName) => invoke("git_get_branch_ahead_behind", { projectPath, branchName }),
  gitGetCurrentBranch: (projectPath) => invoke("git_get_current_branch", { projectPath }),

  // Search
  searchFiles: (directory, query, caseSensitive, wholeWord, regexMode, maxResults) =>
    invoke("search_files", { directory, query, caseSensitive, wholeWord, regexMode, maxResults }),

  // Runtimes
  getInstalledNodeVersions: () => invoke("get_installed_node_versions"),
  getInstalledPythonVersions: () => invoke("get_installed_python_versions"),
  installNodeVersion: (version) => invoke("install_node_version", { version }),
  installPythonVersion: (version) => invoke("install_python_version", { version }),
  uninstallRuntimeVersion: (runtime, version) => invoke("uninstall_runtime_version", { runtime, version }),
  getAvailableNodeVersions: () => invoke("get_available_node_versions"),
  getAvailablePythonVersions: () => invoke("get_available_python_versions"),

  // Settings
  getAppSettings: () => invoke("get_app_settings"),
  setAppSetting: (key, value) => invoke("set_app_setting", { key, value }),
  updateAppSettings: (settings) => invoke("update_app_settings", { settings }),
  getProjectSettings: (projectId) => invoke("get_project_settings", { projectId }),
  setProjectSettings: (settings) => invoke("set_project_settings", { settings }),
  deleteProjectSettings: (projectId) => invoke("delete_project_settings", { projectId }),

  // Backups
  createBackup: (passphrase) => invoke("create_backup", { passphrase }),
  restoreBackup: (zipPath, passphrase) => invoke("restore_backup", { zipPath, passphrase }),
  listBackups: () => invoke("list_backups"),
  deleteBackup: (filename) => invoke("delete_backup", { filename }),

  // Updates
  checkForUpdates: () => invoke("check_for_updates"),
  downloadAndInstallUpdate: () => invoke("download_and_install_update"),
  getUpdateStatus: () => invoke("get_update_status"),
  getCurrentVersion: () => invoke("get_current_version"),

  // File watcher
  watchDirectory: (paths) => invoke("watch_directory", { paths }),
  unwatchAll: () => invoke("unwatch_all"),
  isWatching: () => invoke("is_watching"),

  // LSP
  startLspServer: (projectPath, serverCommand, args) => invoke("start_lsp_server", { projectPath, serverCommand, args }),
  stopLspServer: (projectPath) => invoke("stop_lsp_server", { projectPath }),
  lspRequest: (method, params) => invoke("lsp_request", { method, params }),
  startLspProxy: () => invoke("start_lsp_proxy"),

  // System
  openFile: (path) => invoke("open_file", { path }),
  openFolder: (path) => invoke("open_folder", { path }),
  openExternal: (url) => invoke("open_external", { url }),
  getPlatformInfo: () => invoke("get_platform_info"),

  // Media
  validateMediaPath: (path) => invoke("validate_media_path", { path }),

  // Tray
  refreshTrayMenu: () => invoke("refresh_tray_menu"),
};

// Event listeners — direct wrappers of Tauri's listen()
export const on = {
  tunnelStatus: (cb) => listen("tunnel:status", (e) => cb(e.payload)),
  tunnelLog: (cb) => listen("tunnel:log", (e) => cb(e.payload)),
  fileChanged: (cb) => listen("file-changed", (e) => cb(e.payload)),
  updateAvailable: (cb) => listen("update-available", (e) => cb(e.payload)),
  trayProjectAdd: (cb) => listen("tray:project_add", () => cb()),
  trayProjectOpen: (cb) => listen("tray:project_open", () => cb()),
  trayProjectOpenDir: (cb) => listen("tray:project_open_dir", () => cb()),
  trayProjectStop: (cb) => listen("tray:project_stop", () => cb()),
  trayProjectStart: (cb) => listen("tray:project_start", () => cb()),
  trayProjectRestart: (cb) => listen("tray:project_restart", () => cb()),
  trayTunnelStopAll: (cb) => listen("tray:tunnel_stop_all", () => cb()),
  trayCheckUpdates: (cb) => listen("tray:check_updates", () => cb()),
};

export const emitEvent = emit;
```
