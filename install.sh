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

# Setup shell completion
setup_completion() {
    local shell="$1"
    local completion_dir="$2"
    local completion_file="$3"
    
    mkdir -p "$completion_dir"
    
    if "$BINARY_PATH" completion "$shell" > "$completion_file"; then
        echo "✅ Installed $shell completion to $completion_file"
        return 0
    else
        echo "⚠️  Failed to generate $shell completion"
        return 1
    fi
}

# Detect shell more reliably
CURRENT_SHELL=$(basename "$SHELL")
# Add to PATH if needed
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
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


case "$CURRENT_SHELL" in
    zsh)
        if [ -d "$HOME/.local/share/zsh/site-functions" ]; then
            setup_completion zsh "$HOME/.local/share/zsh/site-functions" "$HOME/.local/share/zsh/site-functions/_rulesify"
        elif [ -d "$HOME/.oh-my-zsh/completions" ]; then
            setup_completion zsh "$HOME/.oh-my-zsh/completions" "$HOME/.oh-my-zsh/completions/_rulesify"
        else
            # Fallback to sourcing from profile
            COMPLETION_FILE="$HOME/.rulesify-completion.zsh"
            if setup_completion zsh "$(dirname "$COMPLETION_FILE")" "$COMPLETION_FILE"; then
                if ! grep -q "source.*rulesify-completion.zsh" "$HOME/.zshrc" 2>/dev/null; then
                    echo "source $COMPLETION_FILE" >> "$HOME/.zshrc"
                    echo "Added completion source to ~/.zshrc"
                fi
            fi
        fi
		echo "📚 To enable tab completion in your current session:"
        echo "  source ~/.zshrc   # or restart your terminal"
        ;;
    bash)
        if [ -d "$HOME/.local/share/bash-completion/completions" ]; then
            setup_completion bash "$HOME/.local/share/bash-completion/completions" "$HOME/.local/share/bash-completion/completions/rulesify"
        elif [ -d "$HOME/.bash_completion.d" ]; then
            setup_completion bash "$HOME/.bash_completion.d" "$HOME/.bash_completion.d/rulesify"
        else
            # Fallback to sourcing from profile
            COMPLETION_FILE="$HOME/.rulesify-completion.bash"
            if setup_completion bash "$(dirname "$COMPLETION_FILE")" "$COMPLETION_FILE"; then
                if ! grep -q "source.*rulesify-completion.bash" "$HOME/.bashrc" 2>/dev/null; then
                    echo "source $COMPLETION_FILE" >> "$HOME/.bashrc"
                    echo "Added completion source to ~/.bashrc"
                fi
            fi
        fi
		echo "📚 To enable tab completion in your current session:"
        echo "  source ~/.bashrc  # or restart your terminal"
        ;;
    fish)
        FISH_COMPLETION_DIR="$HOME/.config/fish/completions"
        setup_completion fish "$FISH_COMPLETION_DIR" "$FISH_COMPLETION_DIR/rulesify.fish"
        echo "✅ Fish completion is automatically loaded."
        ;;
    *)
        echo "ℹ️  Shell completion not automatically configured for $CURRENT_SHELL"
        echo "  Generate completion: rulesify completion $CURRENT_SHELL"
        ;;
esac

echo ""
echo "Done! Run 'rulesify --help' to get started."
