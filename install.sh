#!/bin/bash

# code_prompt.rs installer
# This script installs the latest version of code_prompt to your system

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Determine OS and architecture
detect_os() {
  OS="$(uname -s)"
  ARCH="$(uname -m)"
  
  case "$OS" in
    Linux)
      OS="linux"
      ;;
    Darwin)
      OS="macos"
      ;;
    MINGW*|MSYS*|CYGWIN*)
      OS="windows"
      ;;
    *)
      echo -e "${RED}Unsupported operating system: $OS${NC}"
      exit 1
      ;;
  esac

  case "$ARCH" in
    x86_64)
      ARCH="amd64"
      ;;
    arm64|aarch64)
      ARCH="arm64"
      ;;
    riscv64)
      if [ "$OS" = "linux" ]; then
        ARCH="riscv64"
      else
        echo -e "${RED}RISC-V is only supported on Linux${NC}"
        exit 1
      fi
      ;;
    *)
      echo -e "${RED}Unsupported architecture: $ARCH${NC}"
      exit 1
      ;;
  esac

  # Windows has .exe extension
  EXT=""
  if [ "$OS" = "windows" ]; then
    EXT=".exe"
  fi
  
  BINARY_NAME="code_prompt-${OS}-${ARCH}${EXT}"
}

# Get the latest release URL from GitHub
get_latest_release() {
  echo "Fetching latest release of code_prompt..."
  
  # This requires curl
  if ! command -v curl &> /dev/null; then
    echo -e "${RED}curl is required but not installed. Please install curl and try again.${NC}"
    exit 1
  fi
  
  # Get latest release tag
  GITHUB_API_URL="https://api.github.com/repos/lollipopkit/code_prompt.rs/releases/latest"
  LATEST_TAG=$(curl -s $GITHUB_API_URL | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4)
  
  if [ -z "$LATEST_TAG" ]; then
    echo -e "${RED}Failed to get latest release tag.${NC}"
    exit 1
  fi
  
  DOWNLOAD_URL="https://github.com/lollipopkit/code_prompt.rs/releases/download/${LATEST_TAG}/${BINARY_NAME}"
  
  echo "Latest release: ${LATEST_TAG}"
  echo "Binary: ${BINARY_NAME}"
}

# Download the binary
download_binary() {
  echo "Downloading ${BINARY_NAME}..."
  
  mkdir -p /tmp/code_prompt_install
  TEMP_FILE="/tmp/code_prompt_install/${BINARY_NAME}"
  
  if curl -L -o "$TEMP_FILE" "$DOWNLOAD_URL"; then
    echo -e "${GREEN}Download successful!${NC}"
  else
    echo -e "${RED}Failed to download ${DOWNLOAD_URL}${NC}"
    exit 1
  fi
}

# Install the binary
install_binary() {
  echo ""
  
  # Determine install location
  INSTALL_DIR=""
  if [ -d "$HOME/.local/bin" ] && [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
    INSTALL_DIR="$HOME/.local/bin"
  elif [ -d "/usr/local/bin" ] && [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
  elif [ -n "$HOMEBREW_PREFIX" ] && [ -d "$HOMEBREW_PREFIX/bin" ] && [ -w "$HOMEBREW_PREFIX/bin" ]; then
    INSTALL_DIR="$HOMEBREW_PREFIX/bin"
  elif [ -d "$HOME/bin" ] && [[ ":$PATH:" == *":$HOME/bin:"* ]]; then
    INSTALL_DIR="$HOME/bin"
  else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    echo "You may need to add this directory to your PATH by adding this to your profile:"
    echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
  fi
  
  INSTALL_PATH="${INSTALL_DIR}/code_prompt"
  if [ "$OS" = "windows" ]; then
    INSTALL_PATH="${INSTALL_PATH}.exe"
  fi
  
  # Copy the binary and make it executable
  cp "$TEMP_FILE" "$INSTALL_PATH"
  chmod +x "$INSTALL_PATH"

  echo -e "${GREEN}Installed code_prompt to ${INSTALL_PATH}${NC}"
}

# Verify installation
verify_installation() {
  if command -v code_prompt &> /dev/null; then
    echo "Try running: code_prompt --help"
  else
    echo -e "${RED}Installation may have succeeded, but code_prompt was not found in your PATH.${NC}"
    echo "You might need to:"
    echo "  1. Add ${INSTALL_DIR} to your PATH"
    echo "  2. Restart your terminal or source your shell configuration file"
    echo "  3. Try running: ${INSTALL_PATH}"
  fi
}

# Clean up temporary files
cleanup() {
  echo "Cleaning up..."
  rm -rf /tmp/code_prompt_install
}

# Main function
main() {
  detect_os
  get_latest_release
  download_binary
  install_binary
  cleanup
  verify_installation
}

main
