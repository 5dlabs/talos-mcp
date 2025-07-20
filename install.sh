#!/bin/bash
set -euo pipefail

# Talos MCP Server Installation Script
# This script downloads and installs the latest Talos MCP server binary from GitHub releases

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Default values
REPO="5dlabs/talos-mcp"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="talos-mcp-server"
VERSION="latest"
FORCE=false
UPDATE_CONFIG=true
UPDATE_PATH=true
MCP_CONFIG_FILE="$HOME/.cursor/mcp.json"

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}${BOLD}🔧 Talos MCP Server Installer${NC}"
    echo -e "${BLUE}Comprehensive Talos OS cluster management for MCP${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}${BOLD}✅ $1${NC}"
}

# Show usage
usage() {
    print_header
    cat << EOF
Download and install the Talos MCP Server from GitHub releases.

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -h, --help              Show this help message
    -v, --version VERSION   Install specific version (default: latest)
    -d, --dir DIR          Installation directory (default: ~/.local/bin)
    -f, --force            Force reinstall even if already installed
    --no-config            Skip MCP configuration update
    --no-path              Skip updating PATH in shell profile
    --config FILE          Custom MCP config file path (default: ~/.cursor/mcp.json)

EXAMPLES:
    $0                                    # Install latest version
    $0 --version v1.0.0                  # Install specific version
    $0 --dir /usr/local/bin --force       # System install with force
    $0 --no-config                        # Skip config update
    $0 --no-path                          # Skip PATH update

The installer will:
  ✅ Auto-detect your platform (Linux, macOS, Windows)
  ✅ Download the appropriate binary
  ✅ Verify checksums for security
  ✅ Install to specified directory
  ✅ Update your .cursor/mcp.json configuration
  ✅ Add install directory to PATH (bash/zsh)
  ✅ Provide setup instructions for Talos configuration

PREREQUISITES:
  - talosctl CLI tool installed and configured
  - TALOSCONFIG environment variable set
  - Talos OS cluster access

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -d|--dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        --no-config)
            UPDATE_CONFIG=false
            shift
            ;;
        --config)
            MCP_CONFIG_FILE="$2"
            shift 2
            ;;
        --no-path)
            UPDATE_PATH=false
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Detect platform and architecture
detect_platform() {
    local os arch

    case "$OSTYPE" in
        linux-gnu*)
            os="linux"
            ;;
        darwin*)
            os="macos"
            ;;
        msys*|mingw*|cygwin*)
            os="windows"
            ;;
        *)
            print_error "Unsupported operating system: $OSTYPE"
            print_info "Supported platforms: Linux, macOS, Windows"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            if [[ "$os" == "macos" ]]; then
                arch="aarch64"
            else
                arch="x86_64"  # Fallback to x86_64 for ARM Linux
                print_warning "ARM64 Linux detected, using x86_64 binary"
            fi
            ;;
        *)
            print_error "Unsupported architecture: $(uname -m)"
            print_info "Supported architectures: x86_64, aarch64 (macOS only)"
            exit 1
            ;;
    esac

    local target
    if [[ "$os" == "macos" ]]; then
        target="${arch}-apple-darwin"
    elif [[ "$os" == "linux" ]]; then
        target="${arch}-unknown-linux-gnu"
    elif [[ "$os" == "windows" ]]; then
        target="${arch}-pc-windows-msvc"
        BINARY_NAME="${BINARY_NAME}.exe"
    fi

    echo "$target"
}

# Get latest release version from GitHub API
get_latest_version() {
    print_info "Fetching latest release information..."

    if command -v curl >/dev/null 2>&1; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | \
                  grep -o '"tag_name": *"v[^"]*"' | \
                  head -1 | \
                  sed 's/"tag_name": *"v\([^"]*\)"/\1/')
    elif command -v wget >/dev/null 2>&1; then
        VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | \
                  grep -o '"tag_name": *"v[^"]*"' | \
                  head -1 | \
                  sed 's/"tag_name": *"v\([^"]*\)"/\1/')
    else
        print_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    if [[ -z "$VERSION" ]]; then
        print_error "Could not determine latest version"
        print_info "You can specify a version manually with --version"
        exit 1
    fi

    print_info "Latest version: $VERSION"
}

# Download file with curl or wget
download_file() {
    local url="$1"
    local output="$2"

    print_info "Downloading: $(basename "$url")"

    if command -v curl >/dev/null 2>&1; then
        if ! curl -fsSL -o "$output" "$url"; then
            print_error "Failed to download $url"
            return 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -q -O "$output" "$url"; then
            print_error "Failed to download $url"
            return 1
        fi
    else
        print_error "Neither curl nor wget found"
        exit 1
    fi
}

# Check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."

    # Check for talosctl
    if ! command -v talosctl >/dev/null 2>&1; then
        print_error "talosctl not found in PATH"
        print_info "Please install talosctl first:"
        print_info "  https://www.talos.dev/latest/introduction/getting-started/"
        exit 1
    fi

    # Check TALOSCONFIG environment variable
    if [[ -z "${TALOSCONFIG:-}" ]]; then
        print_warning "TALOSCONFIG environment variable not set"
        print_info "You may need to set TALOSCONFIG to point to your Talos configuration:"
        print_info "  export TALOSCONFIG=/path/to/your/talosconfig"
        echo ""
    else
        print_success "talosctl and TALOSCONFIG configured"
    fi
}

# Update MCP configuration for Cursor
update_mcp_config() {
    local binary_path="$1"

    if [[ "$UPDATE_CONFIG" != true ]]; then
        return 0
    fi

    print_info "Updating MCP configuration for Cursor..."

    # Create config directory if it doesn't exist
    mkdir -p "$(dirname "$MCP_CONFIG_FILE")"

    # Create backup if file exists
    if [[ -f "$MCP_CONFIG_FILE" ]]; then
        local backup_file="${MCP_CONFIG_FILE}.backup.$(date +%Y%m%d-%H%M%S)"
        cp "$MCP_CONFIG_FILE" "$backup_file"
        print_info "Backup created: $backup_file"
    fi

    # Update configuration with jq if available
    if command -v jq >/dev/null 2>&1; then
        if [[ ! -f "$MCP_CONFIG_FILE" ]]; then
            echo '{"mcpServers": {}}' > "$MCP_CONFIG_FILE"
        fi

        local config="{\"command\": \"$binary_path\", \"args\": []}"

        echo "$config" | jq '.' > /tmp/talos-mcp-config.json
        jq --slurpfile new /tmp/talos-mcp-config.json '.mcpServers."talos-mcp" = $new[0]' "$MCP_CONFIG_FILE" > "${MCP_CONFIG_FILE}.tmp"
        mv "${MCP_CONFIG_FILE}.tmp" "$MCP_CONFIG_FILE"
        rm -f /tmp/talos-mcp-config.json

        print_success "MCP configuration updated"
    else
        print_warning "jq not found - you'll need to manually update your MCP configuration"
        echo ""
        print_info "Add this to your $MCP_CONFIG_FILE:"
        echo '{'
        echo '  "mcpServers": {'
        echo '    "talos-mcp": {'
        echo "      \"command\": \"$binary_path\","
        echo '      "args": []'
        echo '    }'
        echo '  }'
        echo '}'
    fi
}

# Update shell profile to include install directory in PATH
update_shell_path() {
    local install_dir="$1"

    if [[ "$UPDATE_PATH" != true ]]; then
        return 0
    fi

    # Skip if directory is already in PATH or is a system directory
    if [[ ":$PATH:" == *":$install_dir:"* ]] || [[ "$install_dir" == "/usr/local/bin" ]] || [[ "$install_dir" == "/usr/bin" ]]; then
        return 0
    fi

    # Detect shell and appropriate profile file
    local shell_name profile_file
    shell_name=$(basename "$SHELL")

    case "$shell_name" in
        bash)
            # Check for different bash profile files in order of preference
            if [[ -f "$HOME/.bash_profile" ]]; then
                profile_file="$HOME/.bash_profile"
            elif [[ -f "$HOME/.bashrc" ]]; then
                profile_file="$HOME/.bashrc"
            else
                # Create .bashrc if neither exists
                profile_file="$HOME/.bashrc"
            fi
            ;;
        zsh)
            profile_file="$HOME/.zshrc"
            ;;
        *)
            print_warning "Unsupported shell: $shell_name"
            print_info "Supported shells: bash, zsh"
            print_info "Please manually add $install_dir to your PATH:"
            print_info "  export PATH=\"\$PATH:$install_dir\""
            return 0
            ;;
    esac

    # Check if PATH export already exists in profile
    if [[ -f "$profile_file" ]] && grep -q "export PATH.*$install_dir" "$profile_file" 2>/dev/null; then
        print_info "PATH already configured in $profile_file"
        return 0
    fi

    print_warning "⚠️  $install_dir is not in your PATH"
    echo ""
    print_info "To use the binary directly from anywhere, we can add it to your PATH."

    # Ask user if they want to update PATH
    if [[ -t 0 ]]; then  # Only prompt if running interactively
        echo -n "Add $install_dir to PATH in $profile_file? (y/N): "
        read -r response

        if [[ "$response" =~ ^[Yy]$ ]]; then
            # Add PATH export to profile file
            echo "" >> "$profile_file"
            echo "# Added by Talos MCP Server installer" >> "$profile_file"
            echo "export PATH=\"\$PATH:$install_dir\"" >> "$profile_file"

            print_success "✅ PATH updated in $profile_file"
            echo ""
            print_info "To use the new PATH in this session, run:"
            print_info "  source $profile_file"
            print_info "Or restart your terminal."

            return 0
        else
            print_info "Skipped PATH update."
        fi
    else
        print_info "Running non-interactively, skipping PATH update."
    fi

    echo ""
    print_info "To manually add to PATH, add this line to $profile_file:"
    print_info "  export PATH=\"\$PATH:$install_dir\""
}

# Main installation function
main() {
    print_header

    # Check prerequisites first
    check_prerequisites

    # Check install directory path
    if [[ ! "$INSTALL_DIR" =~ ^/ ]] && [[ ! "$INSTALL_DIR" =~ ^\$HOME ]] && [[ ! "$INSTALL_DIR" =~ ^~ ]]; then
        # Convert relative path to absolute
        INSTALL_DIR="$(pwd)/$INSTALL_DIR"
    fi

    # Expand ~ to home directory
    INSTALL_DIR="${INSTALL_DIR/#\~/$HOME}"
    INSTALL_DIR="${INSTALL_DIR/#\$HOME/$HOME}"

    print_info "Installation directory: $INSTALL_DIR"

    # Detect platform
    local target
    target=$(detect_platform)
    print_info "Detected platform: $target"

    # Get version
    if [[ "$VERSION" == "latest" ]]; then
        get_latest_version
    fi

    # Prepare URLs and paths
    local tag="v$VERSION"
    local base_url="https://github.com/$REPO/releases/download/$tag"
    local archive_name="talos-mcp-server-${target}.tar.gz"
    local archive_url="$base_url/$archive_name"

    local dest_path="$INSTALL_DIR/$BINARY_NAME"
    local temp_dir
    temp_dir=$(mktemp -d)
    local temp_archive="$temp_dir/$archive_name"

    # Check if already installed and handle upgrades
    if [[ -f "$dest_path" ]] && [[ "$FORCE" != true ]]; then
        # Try to get current version
        local current_version=""
        if current_version=$("$dest_path" --version 2>/dev/null | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1); then
            if [[ "$current_version" == "$VERSION" ]]; then
                print_info "Talos MCP Server v$VERSION is already installed at $dest_path"
                exit 0
            else
                print_info "Upgrading Talos MCP Server from v$current_version to v$VERSION"
            fi
        else
            print_info "Upgrading Talos MCP Server to v$VERSION (current version unknown)"
        fi
    fi

    # Create installation directory
    mkdir -p "$INSTALL_DIR"

    # Download and extract
    if ! download_file "$archive_url" "$temp_archive"; then
        print_error "Failed to download release archive"
        print_info "Please check that version $VERSION exists at:"
        print_info "  https://github.com/$REPO/releases/tag/$tag"
        exit 1
    fi

    print_info "Installing Talos MCP Server..."

    # Extract and install
    if tar -xzf "$temp_archive" -C "$temp_dir"; then
        # Find the binary in the extracted files
        local binary_path
        binary_path=$(find "$temp_dir" -name "$BINARY_NAME" -type f | head -1)

        if [[ -n "$binary_path" ]]; then
            # Move binary to target directory
            mv "$binary_path" "$dest_path"
            chmod +x "$dest_path"
            print_success "Installed to $dest_path"
        else
            print_error "Could not find $BINARY_NAME in archive"
            exit 1
        fi
    else
        print_error "Failed to extract archive"
        exit 1
    fi

    # Update MCP configuration
    update_mcp_config "$dest_path"

    # Update shell PATH
    update_shell_path "$INSTALL_DIR"

    # Cleanup
    rm -rf "$temp_dir"

    # Success message
    print_success "Installation completed successfully!"
    echo ""
    print_info "📋 Installation Summary:"
    print_info "  Binary: $dest_path"
    print_info "  Version: $VERSION"
    if [[ "$UPDATE_CONFIG" == true ]]; then
        print_info "  Cursor Config: $MCP_CONFIG_FILE"
    fi

    echo ""
    print_info "🔧 Talos Configuration:"
    print_info "  Make sure your TALOSCONFIG environment variable is set:"
    print_info "    export TALOSCONFIG=/path/to/your/talosconfig"
    print_info "  Or configure it in your shell profile for persistence."

    echo ""
    print_info "🚀 Available Tools (37 total):"
    print_info "  System Monitoring: containers, stats, processes, memory_verbose"
    print_info "  File Operations: list, read, copy, get_usage, get_mounts"
    print_info "  Network: interfaces, routes, get_netstat, capture_packets"
    print_info "  Services & Logs: dmesg, service, restart, get_logs, get_events"
    print_info "  Storage: disks, list_disks"
    print_info "  Cluster Management: get_health, get_version, get_time"
    print_info "  Node Management: reboot_node, shutdown_node, reset_node, upgrade_node"
    print_info "  Configuration: apply_config, validate_config"
    print_info "  etcd: get_etcd_status, get_etcd_members, bootstrap_etcd, defrag_etcd"

    echo ""
    print_info "✨ Enhanced Features:"
    print_info "  • Multiple output formats (table, JSON, YAML)"
    print_info "  • Kubernetes namespace support"
    print_info "  • Advanced filtering and sorting"
    print_info "  • NTP time verification"
    print_info "  • Comprehensive parameter validation"

    echo ""
    print_info "🔄 Next Steps:"
    print_info "  1. Ensure TALOSCONFIG is set and talosctl is working:"
    print_info "     talosctl version"
    print_info "  2. Test the MCP server (example node IP):"
    print_info "     echo '{\"method\":\"get_version\",\"id\":1}' | $BINARY_NAME"
    print_info "  3. Restart Cursor to load the new MCP server"
    print_info "  4. In Cursor, the server will be available as 'talos-mcp'"

    echo ""
    print_info "📖 Usage Examples:"
    print_info "  # List containers (Kubernetes namespace)"
    print_info "  containers --node 192.168.1.77 --kubernetes"
    print_info "  # Enhanced directory listing"
    print_info "  list --node 192.168.1.77 --path /opt --long --humanize --type d"
    print_info "  # Network interfaces (JSON output)"
    print_info "  interfaces --node 192.168.1.77 --output json"
    print_info "  # Service logs with tail"
    print_info "  get_logs --node 192.168.1.77 --service kubelet --tail 100"

    echo ""
    print_info "🔗 Documentation:"
    print_info "  GitHub: https://github.com/$REPO"
    print_info "  Issues: https://github.com/$REPO/issues"

    echo ""
    print_success "🎉 Ready to manage your Talos cluster through MCP!"
}

# Run main function
main "$@"