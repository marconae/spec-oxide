#!/usr/bin/env bash
#
# Spox MCP Setup Script
# =====================
#
# This script helps configure MCP servers for use with Claude Code in a Spox project.
# It manages the .mcp.json file directly, keeping configuration project-scoped.
#
# Supported platforms: Linux, macOS
# Windows users: Run this script in WSL (Windows Subsystem for Linux)
#
# Usage: .spox/setup.sh
#
# The script is interactive and will ask for confirmation before each step.
# It is idempotent - safe to run multiple times.
#
# Prerequisites:
#   - jq (for JSON manipulation)
#   - uv (for Serena MCP, optional)
#

set -euo pipefail

# =============================================================================
# Color and Output Helpers
# =============================================================================

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Print functions for consistent output formatting
print_header() {
    echo ""
    echo -e "${BOLD}${CYAN}$1${NC}"
    echo -e "${CYAN}$(printf '=%.0s' $(seq 1 ${#1}))${NC}"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}!${NC} $1"
}

print_info() {
    echo -e "${CYAN}→${NC} $1"
}

# =============================================================================
# Utility Functions
# =============================================================================

# Check if a command exists in PATH
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Prompt user for yes/no confirmation
# Returns 0 for yes, 1 for no
confirm() {
    local prompt="$1"
    local response

    while true; do
        echo -en "${BOLD}$prompt${NC} [y/n]: "
        read -r response
        case "$response" in
            [yY]|[yY][eE][sS])
                return 0
                ;;
            [nN]|[nN][oO])
                return 1
                ;;
            *)
                echo "Please answer y or n."
                ;;
        esac
    done
}

# =============================================================================
# Platform Check
# =============================================================================

check_platform() {
    local os
    os="$(uname -s)"

    case "$os" in
        Linux|Darwin)
            print_success "Platform: $os (supported)"
            return 0
            ;;
        MINGW*|MSYS*|CYGWIN*)
            print_error "Native Windows is not supported."
            print_info "Please run this script in WSL (Windows Subsystem for Linux)."
            print_info "Install WSL: https://learn.microsoft.com/en-us/windows/wsl/install"
            exit 1
            ;;
        *)
            print_warning "Unknown platform: $os"
            print_info "This script is designed for Linux and macOS."
            if ! confirm "Continue anyway?"; then
                exit 1
            fi
            ;;
    esac
}

# =============================================================================
# Prerequisite Checks
# =============================================================================

check_jq() {
    print_header "Checking jq"

    if command_exists jq; then
        print_success "jq is installed"
        return 0
    else
        print_error "jq is not installed (required for JSON manipulation)"
        print_info "Install with one of these methods:"
        echo "    macOS:  brew install jq"
        echo "    Ubuntu: sudo apt-get install jq"
        echo "    Fedora: sudo dnf install jq"
        return 1
    fi
}

check_uv() {
    print_header "Checking uv (for Serena MCP)"

    if command_exists uv; then
        print_success "uv is installed"
        return 0
    else
        print_warning "uv is not installed (required for Serena MCP)"
        print_info "Install with:"
        echo "    curl -LsSf https://astral.sh/uv/install.sh | sh"
        return 1
    fi
}

# =============================================================================
# MCP JSON Configuration Functions
# =============================================================================

MCP_JSON_FILE=".mcp.json"

# Check if an MCP server is configured in .mcp.json
# Returns 0 if configured, 1 if not
is_mcp_configured() {
    local name="$1"
    if [ -f "$MCP_JSON_FILE" ]; then
        jq -e ".mcpServers.\"$name\"" "$MCP_JSON_FILE" >/dev/null 2>&1
    else
        return 1
    fi
}

# Get the spox MCP server configuration JSON
get_spox_config() {
    cat <<'EOF'
{"command": "spox", "args": ["mcp", "serve"]}
EOF
}

# Get the serena MCP server configuration JSON
get_serena_config() {
    cat <<'EOF'
{"command": "uvx", "args": ["--from", "git+https://github.com/oraios/serena", "serena", "start-mcp-server", "--context", "claude-code", "--project", "."]}
EOF
}

# Get the context7 MCP server configuration JSON
get_context7_config() {
    cat <<'EOF'
{"type": "http", "url": "https://mcp.context7.com/mcp"}
EOF
}

# Initialize .mcp.json if it doesn't exist
init_mcp_json() {
    if [ ! -f "$MCP_JSON_FILE" ]; then
        echo '{"mcpServers": {}}' > "$MCP_JSON_FILE"
        print_success "Created $MCP_JSON_FILE"
    fi
}

# Add or update an MCP server in .mcp.json
# Usage: set_mcp_server <name> <config_json>
set_mcp_server() {
    local name="$1"
    local config="$2"
    local temp_file
    temp_file=$(mktemp)

    jq --argjson config "$config" ".mcpServers.\"$name\" = \$config" "$MCP_JSON_FILE" > "$temp_file"
    mv "$temp_file" "$MCP_JSON_FILE"
}

# Ensure .mcp.json is properly configured
ensure_mcp_json() {
    print_header "Configuring .mcp.json"

    # Initialize file if needed
    init_mcp_json

    # Always add/update spox (ensures latest config)
    print_info "Configuring spox MCP server..."
    set_mcp_server "spox" "$(get_spox_config)"
    print_success "spox MCP configured"

    # Add serena if not present (with confirmation)
    if ! is_mcp_configured "serena"; then
        echo ""
        print_info "Serena provides semantic code navigation and understanding."
        if confirm "Add Serena MCP to configuration?"; then
            set_mcp_server "serena" "$(get_serena_config)"
            print_success "serena MCP configured"
        else
            print_warning "Skipping Serena MCP configuration"
            print_info "You can add it later by running this script again."
        fi
    else
        print_success "serena MCP already configured (unchanged)"
    fi

    # Add context7 if not present (with confirmation)
    if ! is_mcp_configured "context7"; then
        echo ""
        print_info "Context7 provides up-to-date library documentation."
        if confirm "Add Context7 MCP to configuration?"; then
            set_mcp_server "context7" "$(get_context7_config)"
            print_success "context7 MCP configured"
        else
            print_warning "Skipping Context7 MCP configuration"
            print_info "You can add it later by running this script again."
        fi
    else
        print_success "context7 MCP already configured (unchanged)"
    fi
}

# =============================================================================
# Serena Project Indexing
# =============================================================================

index_project_with_serena() {
    print_header "Serena Project Indexing"

    print_info "Indexing helps Serena understand your codebase faster."
    print_info "This may take a moment depending on project size."
    echo ""

    if ! confirm "Index this project with Serena?"; then
        print_warning "Skipping project indexing"
        print_info "Serena will index on first use (may be slower initially)."
        return 0
    fi

    print_info "Indexing project..."

    if uvx --from git+https://github.com/oraios/serena serena project index; then
        print_success "Project indexed successfully"
        return 0
    else
        print_error "Failed to index project"
        print_info "Serena will still work, but initial queries may be slower."
        return 1
    fi
}

# =============================================================================
# Main Script
# =============================================================================

main() {
    echo ""
    echo -e "${BOLD}${CYAN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${CYAN}║     Spox MCP Setup                    ║${NC}"
    echo -e "${BOLD}${CYAN}╚═══════════════════════════════════════╝${NC}"
    echo ""
    print_info "This script will help you configure MCP servers for Claude Code."
    print_info "Configuration is stored in .mcp.json (project-scoped)."

    # Check platform compatibility
    check_platform

    # Check required prerequisite
    if ! check_jq; then
        print_error "jq is required but not installed."
        print_info "Install jq first, then run this script again."
        exit 1
    fi

    # Check optional prerequisites
    local has_uv=false
    if check_uv; then
        has_uv=true
    fi

    # Summary of prerequisites
    print_header "Prerequisites Summary"

    if [ "$has_uv" = false ]; then
        print_warning "uv not installed - Serena MCP will require uv to run"
        print_info "Serena can still be configured, but won't work until uv is installed."
    fi

    # Configure .mcp.json
    ensure_mcp_json

    # Offer to index project if serena is configured and uv is available
    if is_mcp_configured "serena" && [ "$has_uv" = true ]; then
        index_project_with_serena
    fi

    # Final summary
    print_header "Setup Complete"
    print_success "MCP configuration finished!"
    echo ""
    print_info "Configuration saved to: $MCP_JSON_FILE"
    echo ""
    print_info "Next steps:"
    echo "  1. Start Claude Code in this directory"
    echo "  2. The MCP servers will be discovered automatically from .mcp.json"
    echo ""
    print_info "To view current configuration:"
    echo "  cat .mcp.json | jq"
    echo ""
}

# Run main function
main "$@"
