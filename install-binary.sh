#!/usr/bin/env bash

# tmux-copyrat binary installer
# This script automatically downloads the appropriate pre-built binary
# from GitHub releases based on the detected system architecture.

set -euo pipefail

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Configuration
REPO_OWNER="graelo"
REPO_NAME="tmux-copyrat"
BINARY_NAME="tmux-copyrat"

# Function to log messages with timestamp
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-copyrat-installer: $1" >&2
}

# Function to detect system OS
detect_os() {
    case "$(uname -s)" in
        Darwin) echo "darwin" ;;
        Linux) echo "linux" ;;
        *) echo "unknown" ;;
    esac
}

# Function to detect system architecture
detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64|amd64) echo "x86_64" ;;
        arm64|aarch64) echo "aarch64" ;;
        *) echo "unknown" ;;
    esac
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to get the latest release info from GitHub API
get_latest_release_info() {
    local api_url="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest"
    
    if command_exists curl; then
        curl -s "$api_url" 2>/dev/null
    elif command_exists wget; then
        wget -qO- "$api_url" 2>/dev/null
    else
        log_message "Error: Neither curl nor wget is available for downloading"
        return 1
    fi
}

# Function to extract download URL from release info
get_download_url() {
    local release_info="$1"
    local os="$2"
    local arch="$3"
    
    # Map OS and arch to the expected naming convention in releases
    local target_name
    if [[ "$os" == "darwin" ]]; then
        target_name="${BINARY_NAME}-${arch}-apple-darwin.zip"
    elif [[ "$os" == "linux" ]]; then
        target_name="${BINARY_NAME}-${arch}-unknown-linux-musl.tar.xz"
    else
        log_message "Error: Unsupported OS: $os"
        return 1
    fi
    
    # Extract download URL using grep and sed (avoiding jq dependency)
    echo "$release_info" | grep -o "\"browser_download_url\":[[:space:]]*\"[^\"]*${target_name}\"" | sed 's/.*"browser_download_url":[[:space:]]*"\([^"]*\)".*/\1/'
}

# Function to find the binary after extraction (handles both flat and nested structures)
find_extracted_binary() {
    local target_dir="$1"
    local binary_name="$2"
    
    # Look for the binary in common locations
    local search_paths=(
        "${target_dir}/${binary_name}"                    # Flat structure (future releases)
        "${target_dir}/target/release/${binary_name}"     # Current nested structure
        "${target_dir}/release/${binary_name}"            # Alternative nested structure
    )
    
    for path in "${search_paths[@]}"; do
        if [[ -f "$path" ]]; then
            echo "$path"
            return 0
        fi
    done
    
    # If not found in expected locations, search recursively
    local found_binary
    found_binary=$(find "$target_dir" -name "$binary_name" -type f 2>/dev/null | head -1)
    if [[ -n "$found_binary" ]]; then
        echo "$found_binary"
        return 0
    fi
    
    return 1
}

# Function to download and extract binary
download_and_extract_binary() {
    local download_url="$1"
    local target_dir="$2"
    local os="$3"
    
    local filename=$(basename "$download_url")
    local temp_file="${target_dir}/${filename}"
    
    log_message "Downloading binary from: $download_url"
    
    # Download the file
    if command_exists curl; then
        if ! curl -L -o "$temp_file" "$download_url" 2>/dev/null; then
            log_message "Error: Failed to download binary using curl"
            return 1
        fi
    elif command_exists wget; then
        if ! wget -O "$temp_file" "$download_url" 2>/dev/null; then
            log_message "Error: Failed to download binary using wget"
            return 1
        fi
    else
        log_message "Error: Neither curl nor wget is available"
        return 1
    fi
    
    # Extract the binary
    if [[ "$filename" == *.zip ]]; then
        if command_exists unzip; then
            if ! unzip -q "$temp_file" -d "$target_dir" 2>/dev/null; then
                log_message "Error: Failed to extract zip file"
                rm -f "$temp_file"
                return 1
            fi
        else
            log_message "Error: unzip command not available for extracting zip file"
            rm -f "$temp_file"
            return 1
        fi
    elif [[ "$filename" == *.tar.xz ]]; then
        if command_exists tar; then
            if ! tar -xf "$temp_file" -C "$target_dir" 2>/dev/null; then
                log_message "Error: Failed to extract tar.xz file"
                rm -f "$temp_file"
                return 1
            fi
        else
            log_message "Error: tar command not available for extracting tar.xz file"
            rm -f "$temp_file"
            return 1
        fi
    else
        log_message "Error: Unknown archive format: $filename"
        rm -f "$temp_file"
        return 1
    fi
    
    # Clean up the downloaded archive
    rm -f "$temp_file"
    
    # Find the binary (handles both flat and nested structures)
    local extracted_binary_path
    if ! extracted_binary_path=$(find_extracted_binary "$target_dir" "$BINARY_NAME"); then
        log_message "Error: Binary not found after extraction"
        return 1
    fi
    
    # Move binary to the expected location if it's not already there
    local final_binary_path="${target_dir}/${BINARY_NAME}"
    if [[ "$extracted_binary_path" != "$final_binary_path" ]]; then
        if ! mv "$extracted_binary_path" "$final_binary_path" 2>/dev/null; then
            log_message "Error: Failed to move binary to final location"
            return 1
        fi
        
        # Clean up any empty directories left behind
        local extracted_dir=$(dirname "$extracted_binary_path")
        if [[ "$extracted_dir" != "$target_dir" ]] && [[ -d "$extracted_dir" ]]; then
            rmdir "$extracted_dir" 2>/dev/null || true
            # Try to remove parent directories if they're empty
            local parent_dir=$(dirname "$extracted_dir")
            if [[ "$parent_dir" != "$target_dir" ]] && [[ -d "$parent_dir" ]]; then
                rmdir "$parent_dir" 2>/dev/null || true
            fi
        fi
    fi
    
    # Make the binary executable
    if [[ -f "$final_binary_path" ]]; then
        chmod +x "$final_binary_path"
        log_message "Binary downloaded and extracted successfully"
        return 0
    else
        log_message "Error: Binary not found at final location"
        return 1
    fi
}

# Function to check if binary needs updating
should_update_binary() {
    local binary_path="$1"
    
    # If binary doesn't exist, we need to download it
    if [[ ! -f "$binary_path" ]]; then
        return 0
    fi
    
    # If binary exists but is not executable, we need to download it
    if [[ ! -x "$binary_path" ]]; then
        return 0
    fi
    
    # If binary exists and is executable, don't re-download unless forced
    # This is more idiomatic for tmux plugins - simple existence check
    return 1  # Don't update, binary exists and works
}

# Main installation function
install_binary() {
    local binary_path="${SCRIPT_DIR}/${BINARY_NAME}"
    
    # Check if we need to update the binary
    if ! should_update_binary "$binary_path"; then
        log_message "Binary is up-to-date, skipping download"
        return 0
    fi
    
    log_message "Installing tmux-copyrat binary..."
    
    # Detect system information
    local os=$(detect_os)
    local arch=$(detect_arch)
    
    if [[ "$os" == "unknown" ]]; then
        log_message "Error: Unsupported operating system: $(uname -s)"
        log_message "Please install manually from: https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"
        return 1
    fi
    
    if [[ "$arch" == "unknown" ]]; then
        log_message "Error: Unsupported architecture: $(uname -m)"
        log_message "Please install manually from: https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"
        return 1
    fi
    
    log_message "Detected system: $os-$arch"
    
    # Get latest release information
    local release_info
    if ! release_info=$(get_latest_release_info); then
        log_message "Error: Failed to fetch release information from GitHub API"
        log_message "Please check your internet connection or install manually"
        return 1
    fi
    
    # Get the download URL for the appropriate binary
    local download_url
    if ! download_url=$(get_download_url "$release_info" "$os" "$arch"); then
        log_message "Error: Failed to find appropriate binary for $os-$arch"
        log_message "Please install manually from: https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"
        return 1
    fi
    
    if [[ -z "$download_url" ]]; then
        log_message "Error: No download URL found for $os-$arch"
        log_message "Please install manually from: https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"
        return 1
    fi
    
    # Download and extract the binary
    if download_and_extract_binary "$download_url" "$SCRIPT_DIR" "$os"; then
        log_message "Installation completed successfully"
        return 0
    else
        log_message "Error: Failed to download and install binary"
        log_message "Please install manually from: https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"
        return 1
    fi
}

# Show help
show_help() {
    cat << EOF
tmux-copyrat binary installer

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -h, --help     Show this help message
    -f, --force    Force reinstallation even if binary exists
    -q, --quiet    Suppress log messages (errors still shown)

EXAMPLES:
    $0              # Install binary if not present
    $0 --force      # Force reinstall (downloads latest version)
    $0 --quiet      # Install quietly (used by tmux-copyrat.tmux)

NOTE:
    This script follows tmux plugin conventions - it only downloads
    if the binary doesn't exist. Use --force to get the latest version.

EOF
}

# Parse command line arguments
FORCE_INSTALL=false
QUIET_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -f|--force)
            FORCE_INSTALL=true
            shift
            ;;
        -q|--quiet)
            QUIET_MODE=true
            shift
            ;;
        *)
            echo "Unknown option: $1" >&2
            echo "Use --help for usage information" >&2
            exit 1
            ;;
    esac
done

# Override log function for quiet mode
if [[ "$QUIET_MODE" == "true" ]]; then
    log_message() {
        # Only show errors in quiet mode
        if [[ "$1" =~ ^Error: ]]; then
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-copyrat-installer: $1" >&2
        fi
    }
fi

# Force install by removing existing binary
if [[ "$FORCE_INSTALL" == "true" ]]; then
    rm -f "${SCRIPT_DIR}/${BINARY_NAME}"
fi

# Run the installation
if install_binary; then
    exit 0
else
    exit 1
fi