#!/usr/bin/env bash
#
# ugly — Universal Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/USERNAME/ugly/main/install.sh | sh
#
# Supports: Linux (x86_64, aarch64), macOS (x86_64, aarch64)
# Package managers: apt, dnf, pacman, yum, brew
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
        CYGWIN*|MINGW*|MSYS*) OS="windows" ;;
        *) error "Unsupported OS: $os_type" ;;
    esac
}

# ── Detect Architecture ──────────────────────────────────────────
detect_arch() {
    local arch
    arch="$(uname -m)"

    case "$arch" in
        x86_64|amd64)  ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *) error "Unsupported architecture: $arch" ;;
    esac
}

# ── Detect Package Manager ───────────────────────────────────────
detect_package_manager() {
    if command -v apt-get &>/dev/null; then
        PKG_MANAGER="apt"
    elif command -v dnf &>/dev/null; then
        PKG_MANAGER="dnf"
    elif command -v pacman &>/dev/null; then
        PKG_MANAGER="pacman"
    elif command -v yum &>/dev/null; then
        PKG_MANAGER="yum"
    elif command -v brew &>/dev/null; then
        PKG_MANAGER="brew"
    else
        PKG_MANAGER="none"
    fi
}

# ── Check for dependencies ───────────────────────────────────────
check_curl() {
    if ! command -v curl &>/dev/null; then
        info "curl not found, installing..."
        case "$PKG_MANAGER" in
            apt)    sudo apt-get update && sudo apt-get install -y curl ;;
            dnf)    sudo dnf install -y curl ;;
            pacman) sudo pacman -Sy --noconfirm curl ;;
            yum)    sudo yum install -y curl ;;
            brew)   brew install curl ;;
            *)      error "Please install curl manually" ;;
        esac
        success "curl installed"
    fi
}

# ── Determine download URL ───────────────────────────────────────
GITHUB_REPO="9sx77ssl/ugly"
VERSION="latest"

get_latest_version() {
    info "Checking latest release..."
    VERSION="$(curl -fsSL "https://api.github.com/repos/$GITHUB_REPO/releases/latest" \
        | grep -o '"tag_name":"[^"]*"' \
        | head -1 \
        | cut -d'"' -f4)"

    if [ -z "$VERSION" ]; then
        warn "Could not fetch latest version, falling back to v1.0.0"
        VERSION="v1.0.0"
    fi
    success "Latest version: $VERSION"
}

get_download_url() {
    local os_ext="$OS"
    [ "$OS" = "macos" ] && os_ext="macos"

    echo "https://github.com/$GITHUB_REPO/releases/download/$VERSION/ugly-$VERSION-$ARCH-$os_ext.tar.gz"
}

# ── Install directory ────────────────────────────────────────────
INSTALL_DIR="/usr/local/bin"

# ── Download and Install ─────────────────────────────────────────
download_and_install() {
    local url
    url="$(get_download_url)"

    local tmp_dir
    tmp_dir="$(mktemp -d)"
    local tarball="$tmp_dir/ugly.tar.gz"

    info "Downloading $VERSION for $OS ($ARCH)..."
    info "URL: $url"

    if ! curl -fsSL -o "$tarball" "$url" 2>/dev/null; then
        warn "Binary not available for this platform yet. Falling back to building from source..."
        rm -rf "$tmp_dir"
        build_from_source
        return
    fi

    success "Downloaded successfully"

    info "Extracting..."
    tar -xzf "$tarball" -C "$tmp_dir"
    success "Extracted"

    # Find the binary
    local binary
    binary="$(find "$tmp_dir" -name 'ugly' -type f | head -1)"

    if [ -z "$binary" ]; then
        # Maybe it's in a subdirectory
        binary="$(find "$tmp_dir" -type f | head -1)"
    fi

    if [ -z "$binary" ]; then
        error "Could not find binary in archive"
    fi

    info "Installing to $INSTALL_DIR/ugly..."
    sudo cp "$binary" "$INSTALL_DIR/ugly"
    sudo chmod +x "$INSTALL_DIR/ugly"

    rm -rf "$tmp_dir"
    success "Installed to $INSTALL_DIR/ugly"
}

# ── Build from Source (fallback) ─────────────────────────────────
build_from_source() {
    info "Building from source..."

    # Check for Rust
    if ! command -v cargo &>/dev/null; then
        info "Rust not found, installing..."
        if command -v curl &>/dev/null; then
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            . "$HOME/.cargo/env"
        else
            error "Please install Rust first: https://rustup.rs"
        fi
    else
        success "Rust found: $(cargo --version)"
    fi

    # Clone repo
    local tmp_dir
    tmp_dir="$(mktemp -d)"

    info "Cloning repository..."
    git clone --depth 1 --branch "$VERSION" \
        "https://github.com/$GITHUB_REPO.git" "$tmp_dir" 2>/dev/null \
        || git clone --depth 1 "https://github.com/$GITHUB_REPO.git" "$tmp_dir"

    cd "$tmp_dir"

    info "Building release binary..."
    cargo build --release

    info "Installing..."
    sudo cp target/release/ugly "$INSTALL_DIR/ugly"
    sudo chmod +x "$INSTALL_DIR/ugly"

    cd /
    rm -rf "$tmp_dir"

    success "Built and installed from source"
}

# ── Verify installation ──────────────────────────────────────────
verify_install() {
    if command -v ugly &>/dev/null; then
        echo ""
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
        echo -e "${BOLD}🔑 ugly installed successfully!${RESET}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
        echo ""
        ugly --version
        echo ""
        echo -e "  ${CYAN}Usage:${RESET}"
        echo -e "    ${BOLD}ugly generate --pattern moda --threads 8${RESET}"
        echo -e "    ${BOLD}ugly benchmark${RESET}"
        echo -e "    ${BOLD}ugly decrypt --file wallets.enc${RESET}"
        echo ""
        echo -e "  ${CYAN}More info:${RESET} https://github.com/$GITHUB_REPO"
        echo ""
    else
        error "Installation failed — ugly not found in PATH"
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
    detect_package_manager

    info "OS: $OS | Arch: $ARCH | Package manager: $PKG_MANAGER"
    check_curl
    get_latest_version
    download_and_install
    verify_install
}

main "$@"
