#!/usr/bin/env bash
set -euo pipefail

# ╔══════════════════════════════════════════════════════════════════╗
# ║            SelfHost Helper — Full Build Pipeline                ║
# ║  Cross-platform: Linux (deb/AppImage/rpm), macOS (.app/.dmg),  ║
# ║                  Windows (NSIS via cross-compile or native)     ║
# ╚══════════════════════════════════════════════════════════════════╝

# ─── Colors & Formatting ──────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
DIM='\033[2m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# ─── Helpers ───────────────────────────────────────────────────────
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

START_TIME=$(date +%s)
STEP=0
TOTAL_STEPS=5

log()      { echo -e "${BLUE}  ┃${NC} $*"; }
success()  { echo -e "${GREEN}  ✓${NC} $*"; }
warn()     { echo -e "${YELLOW}  ⚠${NC} $*"; }
error()    { echo -e "${RED}  ✗${NC} $*"; }
step()     { STEP=$((STEP + 1)); echo -e "\n${CYAN}━━━ [${STEP}/${TOTAL_STEPS}]${NC} ${BOLD}$*${NC}"; }
divider()  { echo -e "${DIM}  ───────────────────────────────────────────────────────${NC}"; }
elapsed()  {
    local now=$(date +%s)
    local diff=$((now - START_TIME))
    if [ "$diff" -ge 60 ]; then
        printf "%dm %ds" $((diff / 60)) $((diff % 60))
    else
        printf "%ds" "$diff"
    fi
}

# ─── Platform Detection ───────────────────────────────────────────
detect_platform() {
    case "$(uname -s)" in
        Linux*)   PLATFORM="linux";  PLATFORM_NAME="Linux" ;;
        Darwin*)  PLATFORM="macos";  PLATFORM_NAME="macOS" ;;
        MINGW*|MSYS*|CYGWIN*) PLATFORM="windows"; PLATFORM_NAME="Windows" ;;
        *)        PLATFORM="unknown"; PLATFORM_NAME="Unknown" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *)             ARCH="$(uname -m)" ;;
    esac
}

# ─── Dependency Checks ────────────────────────────────────────────
check_deps() {
    local missing=()
    for cmd in npm node cargo; do
        if ! command -v "$cmd" &>/dev/null; then
            missing+=("$cmd")
        fi
    done

    if [ ${#missing[@]} -gt 0 ]; then
        error "Missing required tools: ${missing[*]}"
        error "Install them and try again."
        exit 1
    fi

    if ! command -v tauri &>/dev/null && ! npx tauri --version &>/dev/null 2>&1; then
        warn "Tauri CLI not found globally — will use npx tauri"
        TAURI_CMD="npx tauri"
    else
        TAURI_CMD="tauri"
    fi
}

# ─── Parse Args ────────────────────────────────────────────────────
VERSION_NAME="${1:-local}"
BUNDLE_TYPES=()
SKIP_DEPS=false
DRY_RUN=false
OUTPUT_BASE="${ROOT_DIR}/../SelfHost-Helper-Versions"

usage() {
    echo -e "${BOLD}Usage:${NC} ./build.sh [OPTIONS] [VERSION_NAME]"
    echo ""
    echo -e "${BOLD}Options:${NC}"
    echo "  --only-bund=TYPE    Bundle only specific type (deb, appimage, rpm, nsis, dmg)"
    echo "                      Can be repeated. Default: platform-native set"
    echo "  --output-dir=DIR    Custom output directory"
    echo "  --skip-deps         Skip npm install step"
    echo "  --dry-run           Show what would be built without building"
    echo "  -h, --help          Show this help"
    echo ""
    echo -e "${BOLD}Examples:${NC}"
    echo "  ./build.sh                          # Build with default version name 'local'"
    echo "  ./build.sh v0.41.0                  # Build and tag as v0.41.0"
    echo "  ./build.sh --only-bund=appimage v2  # Build only AppImage, version v2"
    echo "  ./build.sh --output-dir=/tmp/out    # Custom output location"
}

for arg in "$@"; do
    case "$arg" in
        --only-bund=*)
            BUNDLE_TYPES+=("${arg#*=}")
            ;;
        --output-dir=*)
            OUTPUT_BASE="${arg#*=}"
            ;;
        --skip-deps)
            SKIP_DEPS=true
            ;;
        --dry-run)
            DRY_RUN=true
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            error "Unknown option: $arg"
            usage
            exit 1
            ;;
        *)
            VERSION_NAME="$arg"
            ;;
    esac
done

# ─── Banner ────────────────────────────────────────────────────────
echo ""
echo -e "${MAGENTA}  ╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}  ║${NC}  ${BOLD}${WHITE}SelfHost Helper — Build Pipeline${NC}"
echo -e "${MAGENTA}  ║${NC}  ${DIM}Version: ${VERSION_NAME}${NC}"
echo -e "${MAGENTA}  ╚══════════════════════════════════════════════════════════╝${NC}"
echo ""

# ─── Detect Platform ──────────────────────────────────────────────
detect_platform
log "Platform: ${BOLD}${PLATFORM_NAME}${NC} (${ARCH})"
log "Root:     ${DIM}${ROOT_DIR}${NC}"
divider

# ─── Check Dependencies ───────────────────────────────────────────
check_deps

# ─── Resolve Bundle Types ─────────────────────────────────────────
if [ ${#BUNDLE_TYPES[@]} -eq 0 ]; then
    case "$PLATFORM" in
        linux)
            BUNDLE_TYPES=("appimage" "deb" "rpm")
            ;;
        macos)
            BUNDLE_TYPES=("dmg")
            ;;
        windows)
            BUNDLE_TYPES=("nsis")
            ;;
        *)
            warn "Unknown platform — defaulting to deb"
            BUNDLE_TYPES=("deb")
            ;;
    esac
fi

log "Bundles:  ${BOLD}${BUNDLE_TYPES[*]}${NC}"
divider

if $DRY_RUN; then
    echo ""
    log "Dry run — would execute:"
    log "  1. npm install --legacy-peer-deps"
    log "  2. npx vite build"
    log "  3. cargo tauri build --bundles ${BUNDLE_TYPES[*]}"
    log "  4. Copy artifacts to ${OUTPUT_BASE}/${VERSION_NAME}/"
    log "  5. Print summary"
    echo ""
    exit 0
fi

# ─── Step 1: Install Dependencies ────────────────────────────────
step "Installing frontend dependencies..."

if $SKIP_DEPS; then
    warn "Skipped (--skip-deps)"
else
    if [ -f "package-lock.json" ]; then
        log "Lockfile found — using clean install"
        npm ci --legacy-peer-deps 2>&1 | tail -5
    else
        log "No lockfile — running npm install"
        npm install --legacy-peer-deps 2>&1 | tail -5
    fi
    success "Dependencies installed $(elapsed)"
fi

# ─── Step 2: Build Frontend (Vite) ──────────────────────────────
step "Building frontend (Vite)..."

log "Running: npx vite build"
BUILD_OUTPUT=$(npx vite build 2>&1)
echo "$BUILD_OUTPUT" | tail -3

# Verify dist/ exists
if [ ! -d "dist" ] || [ -z "$(ls -A dist 2>/dev/null)" ]; then
    error "Vite build failed — dist/ is empty or missing"
    exit 1
fi

FILE_COUNT=$(find dist -type f | wc -l)
DIR_SIZE=$(du -sh dist | cut -f1)
success "Frontend built: ${FILE_COUNT} files, ${DIR_SIZE} $(elapsed)"

# ─── Step 3: Build Tauri Release Binary ─────────────────────────
step "Building Tauri release binary..."

# Build the bundle flags for cargo tauri build
BUNDLE_FLAGS=""
for bund in "${BUNDLE_TYPES[@]}"; do
    BUNDLE_FLAGS="${BUNDLE_FLAGS} --bundles ${bund}"
done

log "Running: cargo tauri build${BUNDLE_FLAGS}"
log "This may take a while on first build (compiling Rust dependencies)..."
echo ""

TAURI_START=$(date +%s)

if ! cargo tauri build ${BUNDLE_FLAGS}; then
    error "Tauri build failed!"
    echo ""
    error "Common fixes:"
    error "  - Ensure Rust toolchain is installed: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    error "  - Ensure Tauri CLI: cargo install tauri-cli"
    error "  - Check src-tauri/Cargo.toml for dependency issues"
    error "  - On Linux, install system deps: sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf"
    exit 1
fi

TAURI_DURATION=$(( $(date +%s) - TAURI_START ))
TAURI_MINS=$((TAURI_DURATION / 60))
TAURI_SECS=$((TAURI_DURATION % 60))
success "Tauri build complete: ${TAURI_MINS}m ${TAURI_SECS}s $(elapsed)"

# ─── Step 4: Collect Build Artifacts ─────────────────────────────
step "Collecting build artifacts..."

OUTPUT_DIR="${OUTPUT_BASE}/${VERSION_NAME}"
mkdir -p "${OUTPUT_DIR}"

ARTIFACT_COUNT=0

# --- Binary ---
BINARY_PATH="src-tauri/target/release/selfhost-helper"
case "$PLATFORM" in
    windows) BINARY_PATH="${BINARY_PATH}.exe" ;;
esac

if [ -f "$BINARY_PATH" ]; then
    cp "$BINARY_PATH" "${OUTPUT_DIR}/"
    ARTIFACT_COUNT=$((ARTIFACT_COUNT + 1))
    success "Binary: $(basename "$BINARY_PATH") ($(du -h "$BINARY_PATH" | cut -f1))"
else
    warn "Binary not found at ${BINARY_PATH}"
fi

# --- Bundle directory ---
BUNDLE_SRC="src-tauri/target/release/bundle"
if [ -d "$BUNDLE_SRC" ]; then
    cp -r "$BUNDLE_SRC" "${OUTPUT_DIR}/bundle"
    ARTIFACT_COUNT=$((ARTIFACT_COUNT + 1))

    echo ""
    log "Bundle contents:"
    case "$PLATFORM" in
        linux)
            for bund in "${BUNDLE_TYPES[@]}"; do
                case "$bund" in
                    deb)
                        DEB_FILE=$(find "$BUNDLE_SRC/deb" -name "*.deb" 2>/dev/null | head -1)
                        if [ -n "$DEB_FILE" ]; then
                            success "  deb: $(basename "$DEB_FILE") ($(du -h "$DEB_FILE" | cut -f1))"
                        fi
                        ;;
                    appimage)
                        AI_FILE=$(find "$BUNDLE_SRC/appimage" -name "*.AppImage" 2>/dev/null | head -1)
                        if [ -n "$AI_FILE" ]; then
                            success "  AppImage: $(basename "$AI_FILE") ($(du -h "$AI_FILE" | cut -f1))"
                        fi
                        ;;
                    rpm)
                        RPM_FILE=$(find "$BUNDLE_SRC/rpm" -name "*.rpm" 2>/dev/null | head -1)
                        if [ -n "$RPM_FILE" ]; then
                            success "  rpm: $(basename "$RPM_FILE") ($(du -h "$RPM_FILE" | cut -f1))"
                        fi
                        ;;
                esac
            done
            ;;
        macos)
            DMG_FILE=$(find "$BUNDLE_SRC" -name "*.dmg" 2>/dev/null | head -1)
            if [ -n "$DMG_FILE" ]; then
                success "  dmg: $(basename "$DMG_FILE") ($(du -h "$DMG_FILE" | cut -f1))"
            fi
            APP_DIR=$(find "$BUNDLE_SRC" -name "*.app" -type d 2>/dev/null | head -1)
            if [ -n "$APP_DIR" ]; then
                success "  app: $(basename "$APP_DIR")"
            fi
            ;;
        windows)
            NSIS_FILE=$(find "$BUNDLE_SRC/nsis" -name "*.exe" 2>/dev/null | head -1)
            if [ -n "$NSIS_FILE" ]; then
                success "  nsis: $(basename "$NSIS_FILE") ($(du -h "$NSIS_FILE" | cut -f1))"
            fi
            ;;
    esac
else
    warn "No bundle directory found"
fi

# --- Source snapshot ---
log "Copying source snapshot..."
cp -r dist "${OUTPUT_DIR}/dist"
cp -r src "${OUTPUT_DIR}/src"
cp -r src-tauri "${OUTPUT_DIR}/src-tauri"

# Copy metadata files (non-fatal if missing)
for meta_file in package.json vite.config.js AGENTS.md MIGRATION_PLAN.md SECURITY_AUDIT_PLAN.md; do
    [ -f "$meta_file" ] && cp "$meta_file" "${OUTPUT_DIR}/" 2>/dev/null || true
done

divider
success "Artifacts collected: ${ARTIFACT_COUNT} bundle(s) + source snapshot"

# ─── Step 5: Summary ──────────────────────────────────────────────
step "Build summary"

TOTAL_DURATION=$(elapsed)

echo ""
echo -e "${MAGENTA}  ╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}  ║${NC}  ${BOLD}${GREEN}Build Complete!${NC}"
echo -e "${MAGENTA}  ║${NC}"
echo -e "${MAGENTA}  ║${NC}  ${BOLD}Version:${NC}    ${VERSION_NAME}"
echo -e "${MAGENTA}  ║${NC}  ${BOLD}Platform:${NC}   ${PLATFORM_NAME} (${ARCH})"
echo -e "${MAGENTA}  ║${NC}  ${BOLD}Bundles:${NC}    ${BUNDLE_TYPES[*]}"
echo -e "${MAGENTA}  ║${NC}  ${BOLD}Duration:${NC}   ${TOTAL_DURATION}"
echo -e "${MAGENTA}  ║${NC}  ${BOLD}Output:${NC}     ${OUTPUT_DIR}"
echo -e "${MAGENTA}  ║${NC}"

# Show total output size
if [ -d "$OUTPUT_DIR" ]; then
    TOTAL_SIZE=$(du -sh "$OUTPUT_DIR" | cut -f1)
    echo -e "${MAGENTA}  ║${NC}  ${BOLD}Total size:${NC} ${TOTAL_SIZE}"
fi

echo -e "${MAGENTA}  ║${NC}"
echo -e "${MAGENTA}  ║${NC}  ${DIM}Files:${NC}"

if [ -f "${OUTPUT_DIR}/selfhost-helper" ] || [ -f "${OUTPUT_DIR}/selfhost-helper.exe" ]; then
    BIN_NAME="selfhost-helper"
    [ "$PLATFORM" = "windows" ] && BIN_NAME="selfhost-helper.exe"
    echo -e "${MAGENTA}  ║${NC}    ${GREEN}→${NC} Binary:  ${OUTPUT_DIR}/${BIN_NAME}"
fi

case "$PLATFORM" in
    linux)
        [ -d "${OUTPUT_DIR}/bundle/appimage" ] && \
            echo -e "${MAGENTA}  ║${NC}    ${GREEN}→${NC} AppImage: ${OUTPUT_DIR}/bundle/appimage/"
        [ -d "${OUTPUT_DIR}/bundle/deb" ] && \
            echo -e "${MAGENTA}  ║${NC}    ${GREEN}→${NC} Deb:     ${OUTPUT_DIR}/bundle/deb/"
        [ -d "${OUTPUT_DIR}/bundle/rpm" ] && \
            echo -e "${MAGENTA}  ║${NC}    ${GREEN}→${NC} Rpm:     ${OUTPUT_DIR}/bundle/rpm/"
        ;;
    macos)
        [ -d "${OUTPUT_DIR}/bundle" ] && \
            echo -e "${MAGENTA}  ║${NC}    ${GREEN}→${NC} DMG/App: ${OUTPUT_DIR}/bundle/"
        ;;
    windows)
        [ -d "${OUTPUT_DIR}/bundle/nsis" ] && \
            echo -e "${MAGENTA}  ║${NC}    ${GREEN}→${NC} NSIS:    ${OUTPUT_DIR}/bundle/nsis/"
        ;;
esac

echo -e "${MAGENTA}  ║${NC}    ${GREEN}→${NC} Source:  ${OUTPUT_DIR}/src/"
echo -e "${MAGENTA}  ╚══════════════════════════════════════════════════════════╝${NC}"
echo ""
