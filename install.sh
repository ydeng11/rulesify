#!/usr/bin/env bash
set -e

REPO="ydeng11/rulesify"
BINARY_NAME="rulesify"
INSTALL_DIR="$HOME/.local/bin"

# Detect OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

# Map arch names
case "$ARCH" in
    x86_64|amd64)
        ARCH="amd64" ;;
    arm64|aarch64)
        ARCH="aarch64" ;;
    *)
        echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Compose asset name
ASSET="${BINARY_NAME}-${OS}-${ARCH}.tar.gz"

# Get latest release tag from GitHub API
LATEST_TAG=$(curl -s https://api.github.com/repos/$REPO/releases/latest | grep '"tag_name"' | cut -d '"' -f4)
if [ -z "$LATEST_TAG" ]; then
    echo "Could not fetch latest release tag."; exit 1
fi

# Download URL
URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$ASSET"

# Create install dir if needed
mkdir -p "$INSTALL_DIR"

# Download and extract binary
TMPDIR=$(mktemp -d)
echo "Downloading $ASSET from $URL ..."
if ! curl -sSLf "$URL" -o "$TMPDIR/$ASSET"; then
    echo "Failed to download binary. Please check your OS/arch or visit the releases page."; exit 1
fi

# Extract the binary
cd "$TMPDIR"
tar xzf "$ASSET"

# Move and set permissions
mv "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"
rm -rf "$TMPDIR"
echo "Installed $BINARY_NAME to $INSTALL_DIR/$BINARY_NAME"

# Add to PATH if needed
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    PROFILE=""
    if [ -n "$ZSH_VERSION" ]; then
        PROFILE="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        PROFILE="$HOME/.bashrc"
    else
        PROFILE="$HOME/.profile"
    fi
    if ! grep -q "$INSTALL_DIR" "$PROFILE" 2>/dev/null; then
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$PROFILE"
        echo "Added $INSTALL_DIR to PATH in $PROFILE. Please restart your shell or run:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
fi

echo "Done! Run 'rulesify --help' to get started."
