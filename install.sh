#!/usr/bin/env bash
#
# ugly — Universal Binary Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/9sx77ssl/ugly/main/install.sh | sh
#
# Supports: Linux (x86_64), macOS (x86_64, aarch64)
# Downloads pre-built binary from GitHub Releases — no Rust toolchain required.
#

set -euo pipefail

# ── Colors ────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

info()    { echo -e "${CYAN}● $*${RESET}"; }
success() { echo -e "${GREEN}✓ $*${RESET}"; }
warn()    { echo -e "${YELLOW}⚠ $*${RESET}"; }
error()   { echo -e "${RED}✗ $*${RESET}"; exit 1; }

# ── Detect OS ─────────────────────────────────────────────────────
detect_os() {
    local os_type
    os_type="$(uname -s)"

    case "$os_type" in
        Linux*)  OS="linux" ;;
        Darwin*) OS="macos" ;;
        *)       error "Unsupported OS: $os_type\n   Only Linux and macOS are supported." ;;
    esac
}

# ── Detect Architecture ──────────────────────────────────────────
detect_arch() {
    local arch
    arch="$(uname -m)"

    case "$arch" in
        x86_64|amd64)  ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *)             error "Unsupported architecture: $arch" ;;
    esac
}

# ── Check dependencies ───────────────────────────────────────────
check_deps() {
    if ! command -v curl &>/dev/null; then
        error "curl is required but not installed.\n   Please install curl and try again."
    fi
    if ! command -v tar &>/dev/null; then
        error "tar is required but not installed."
    fi
}

# ── GitHub info ──────────────────────────────────────────────────
GITHUB_REPO="9sx77ssl/ugly"
VERSION="latest"

# ── Get latest release tag ───────────────────────────────────────
get_latest_version() {
    info "Checking latest release..."
    VERSION="$(curl -fsSL --max-time 10 "https://api.github.com/repos/$GITHUB_REPO/releases/latest" 2>/dev/null \
        | grep -o '"tag_name":"[^"]*"' \
        | head -1 \
        | cut -d'"' -f4 || true)"

    if [ -z "$VERSION" ]; then
        error "Could not fetch latest release from GitHub.\n   Make sure the repository has at least one release with binaries."
    fi
    success "Latest version: $VERSION"
}

# ── Build download URL ───────────────────────────────────────────
get_download_url() {
    echo "https://github.com/$GITHUB_REPO/releases/download/$VERSION/ugly-$VERSION-$ARCH-$OS.tar.gz"
}

# ── Install directory ────────────────────────────────────────────
INSTALL_DIR="/usr/local/bin"

# ── Download & Install ───────────────────────────────────────────
download_and_install() {
    local url
    url="$(get_download_url)"

    local tmp_dir
    tmp_dir="$(mktemp -d)"
    local tarball="$tmp_dir/ugly.tar.gz"

    info "Downloading $VERSION for $OS ($ARCH)..."

    if ! curl -fsSL --max-time 60 -o "$tarball" "$url" 2>/dev/null; then
        rm -rf "$tmp_dir"
        error "Binary not available for $OS $ARCH yet.\n   Check releases: https://github.com/$GITHUB_REPO/releases\n   Or build from source: cargo build --release"
    fi

    success "Downloaded"
    info "Extracting..."

    tar -xzf "$tarball" -C "$tmp_dir"

    # Find the binary in the archive
    local binary
    binary="$(find "$tmp_dir" -name 'ugly' -type f | head -1)"

    if [ -z "$binary" ]; then
        rm -rf "$tmp_dir"
        error "Could not find 'ugly' binary in archive"
    fi

    info "Installing to $INSTALL_DIR/ugly..."
    sudo cp "$binary" "$INSTALL_DIR/ugly"
    sudo chmod +x "$INSTALL_DIR/ugly"

    rm -rf "$tmp_dir"
    success "Installed to $INSTALL_DIR/ugly"
}

# ── Verify ───────────────────────────────────────────────────────
verify_install() {
    if command -v ugly &>/dev/null; then
        echo ""
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
        echo -e "${BOLD}🔑 ugly installed successfully!${RESET}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
        echo ""
        ugly --version
        echo ""
        echo -e "  ${CYAN}Get started:${RESET}"
        echo -e "    ${BOLD}ugly generate --pattern moda --threads 8${RESET}"
        echo -e "    ${BOLD}ugly benchmark${RESET}"
        echo -e "    ${BOLD}ugly decrypt --file wallets.enc${RESET}"
        echo ""
        echo -e "  ${CYAN}Docs:${RESET} https://github.com/$GITHUB_REPO"
        echo ""
    else
        error "Installation failed — 'ugly' not found in PATH"
    fi
}

# ── Main ─────────────────────────────────────────────────────────
main() {
    echo ""
    echo -e "${BOLD}🔑 ugly — Solana Vanity Address Generator${RESET}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo ""

    detect_os
    detect_arch
    check_deps

    info "OS: $OS | Arch: $ARCH"

    get_latest_version
    download_and_install
    verify_install
}

main "$@"
