#!/usr/bin/env bash
#
# Spox MCP Setup Script
# =====================
#
# This script helps configure MCP servers (Serena and Context7) for use with
# Claude Code in a Spox project.
#
# Supported platforms: Linux, macOS
# Windows users: Run this script in WSL (Windows Subsystem for Linux)
#
# Usage: .spox/setup.sh
#
# The script is interactive and will ask for confirmation before each step.
# It is idempotent - safe to run multiple times.
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

# Get Node.js major version number
get_node_major_version() {
    node --version 2>/dev/null | sed 's/v\([0-9]*\).*/\1/'
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

check_claude_code() {
    print_header "Checking Claude Code"

    if command_exists claude; then
        print_success "Claude Code is installed"
        return 0
    else
        print_error "Claude Code is not installed"
        print_info "Install with one of these methods:"
        echo "    npm install -g @anthropic-ai/claude-code"
        echo "    curl -fsSL https://claude.ai/install.sh | bash"
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

check_node() {
    print_header "Checking Node.js (for Context7 MCP)"

    if command_exists node; then
        local version
        version=$(get_node_major_version)
        if [ "$version" -ge 18 ]; then
            print_success "Node.js v$version is installed (≥18 required)"
            return 0
        else
            print_warning "Node.js v$version is too old (v18+ required)"
            print_info "Update Node.js: https://nodejs.org/"
            return 1
        fi
    else
        print_warning "Node.js is not installed (required for Context7 MCP)"
        print_info "Install from: https://nodejs.org/"
        return 1
    fi
}

# =============================================================================
# MCP Installation Functions
# =============================================================================

# Check if an MCP server is already configured
# Returns 0 if installed, 1 if not
is_mcp_installed() {
    local name="$1"
    claude mcp list 2>/dev/null | grep -q "^$name\$" || \
    claude mcp list 2>/dev/null | grep -q "^$name " || \
    claude mcp list 2>/dev/null | grep -q " $name\$" || \
    claude mcp list 2>/dev/null | grep -q " $name "
}

install_serena_mcp() {
    print_header "Serena MCP Installation"

    # Check if already installed (idempotency)
    if is_mcp_installed "serena"; then
        print_success "Serena MCP is already installed"
        return 0
    fi

    print_info "Serena provides semantic code navigation and understanding."
    print_info "It will be configured for this project directory."
    echo ""

    if ! confirm "Install Serena MCP?"; then
        print_warning "Skipping Serena MCP installation"
        print_info "You can install it later by running this script again."
        return 1
    fi

    print_info "Installing Serena MCP..."

    local output
    output=$(claude mcp add serena -- uvx --from git+https://github.com/oraios/serena serena start-mcp-server --context ide-assistant --project "$(pwd)" 2>&1)
    local exit_code=$?

    # Check for "already exists" message (idempotency)
    if echo "$output" | grep -q "already exists"; then
        print_success "Serena MCP is already installed"
        return 0
    elif [ $exit_code -eq 0 ]; then
        print_success "Serena MCP installed successfully"
        return 0
    else
        print_error "Failed to install Serena MCP"
        print_info "Error: $output"
        print_info "You may need to install it manually. See: https://github.com/oraios/serena"
        return 1
    fi
}

install_context7_mcp() {
    print_header "Context7 MCP Installation"

    # Check if already installed (idempotency)
    if is_mcp_installed "context7"; then
        print_success "Context7 MCP is already installed"
        return 0
    fi

    print_info "Context7 provides up-to-date library documentation."
    echo ""

    if ! confirm "Install Context7 MCP?"; then
        print_warning "Skipping Context7 MCP installation"
        print_info "You can install it later by running this script again."
        return 1
    fi

    print_info "Installing Context7 MCP..."

    local output
    output=$(claude mcp add --transport http context7 https://mcp.context7.com/mcp 2>&1)
    local exit_code=$?

    # Check for "already exists" message (idempotency)
    if echo "$output" | grep -q "already exists"; then
        print_success "Context7 MCP is already installed"
        return 0
    elif [ $exit_code -eq 0 ]; then
        print_success "Context7 MCP installed successfully"
        return 0
    else
        print_error "Failed to install Context7 MCP"
        print_info "Error: $output"
        print_info "You may need to install it manually. See: https://context7.com/docs/installation"
        return 1
    fi
}

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
    print_info "You will be prompted before each installation step."

    # Check platform compatibility
    check_platform

    # Track prerequisites status
    local has_claude=false
    local has_uv=false
    local has_node=false

    # Check all prerequisites
    if check_claude_code; then
        has_claude=true
    fi

    if check_uv; then
        has_uv=true
    fi

    if check_node; then
        has_node=true
    fi

    # Summary of prerequisites
    print_header "Prerequisites Summary"

    if [ "$has_claude" = false ]; then
        print_error "Claude Code is required but not installed."
        print_info "Install Claude Code first, then run this script again."
        exit 1
    fi

    local can_install_serena=true
    local can_install_context7=true

    if [ "$has_uv" = false ]; then
        print_warning "Cannot install Serena MCP (uv not installed)"
        can_install_serena=false
    fi

    if [ "$has_node" = false ]; then
        print_warning "Cannot install Context7 MCP (Node.js 18+ not installed)"
        can_install_context7=false
    fi

    if [ "$can_install_serena" = false ] && [ "$can_install_context7" = false ]; then
        print_error "No MCP servers can be installed due to missing prerequisites."
        print_info "Install the missing prerequisites and run this script again."
        exit 1
    fi

    # Install MCP servers
    local serena_installed=false

    if [ "$can_install_serena" = true ]; then
        if install_serena_mcp; then
            serena_installed=true
        fi
    fi

    if [ "$can_install_context7" = true ]; then
        install_context7_mcp
    fi

    # Index project if Serena was installed
    if [ "$serena_installed" = true ]; then
        index_project_with_serena
    fi

    # Final summary
    print_header "Setup Complete"
    print_success "MCP configuration finished!"
    echo ""
    print_info "Next steps:"
    echo "  1. Start Claude Code in this directory"
    echo "  2. The MCP servers will be available automatically"
    echo ""
    print_info "To verify MCP servers are working:"
    echo "  claude mcp list"
    echo ""
}

# Run main function
main "$@"
