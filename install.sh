#!/usr/bin/env bash
set -e

REPO="ydeng11/rulesify"
BINARY_NAME="rulesify"
INSTALL_DIR="$HOME/.local/bin"
BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"

# Detect OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

# Map arch names
case "$ARCH" in
    x86_64|amd64)
        ARCH="amd64" ;;
    arm64|aarch64)
        ARCH="arm64" ;;
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

echo "Latest available version: $LATEST_TAG"

# Check if binary already exists and get current version
CURRENT_VERSION=""
if [ -x "$BINARY_PATH" ]; then
    # Try to get version from existing binary
    CURRENT_VERSION=$("$BINARY_PATH" --version 2>/dev/null | head -n1 | grep -o 'v[0-9]\+\.[0-9]\+\.[0-9]\+' || echo "")
    if [ -n "$CURRENT_VERSION" ]; then
        echo "Current installed version: $CURRENT_VERSION"

        # Compare versions
        if [ "$CURRENT_VERSION" = "$LATEST_TAG" ]; then
            echo "✅ You already have the latest version ($LATEST_TAG) installed!"
            echo "Re-installing anyway..."
        else
            echo "🔄 Updating from $CURRENT_VERSION to $LATEST_TAG"
        fi
    else
        echo "🔄 Updating existing installation to $LATEST_TAG"
    fi
else
    echo "📦 Installing $BINARY_NAME $LATEST_TAG"
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

# Verify installation
NEW_VERSION=$("$BINARY_PATH" --version 2>/dev/null | head -n1 | grep -o 'v[0-9]\+\.[0-9]\+\.[0-9]\+' || echo "$LATEST_TAG")
echo "✅ Successfully installed $BINARY_NAME $NEW_VERSION to $INSTALL_DIR/$BINARY_NAME"

# Add to PATH if needed
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    # Detect shell more reliably
    CURRENT_SHELL=$(basename "$SHELL")
    PROFILE=""
    case "$CURRENT_SHELL" in
        zsh)
            PROFILE="$HOME/.zshrc"
            ;;
        bash)
            PROFILE="$HOME/.bashrc"
            ;;
        *)
            PROFILE="$HOME/.profile"
            ;;
    esac

    if ! grep -q "$INSTALL_DIR" "$PROFILE" 2>/dev/null; then
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$PROFILE"
        echo "Added $INSTALL_DIR to PATH in $PROFILE. Please restart your shell or run:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
fi

echo "Done! Run 'rulesify --help' to get started."
