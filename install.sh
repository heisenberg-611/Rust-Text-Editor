#!/bin/bash

# Meow Text Editor Installation Script

set -e

echo "Installing Meow Text Editor..."

# 1. Install the binary using Cargo
echo "Running 'cargo install --path .' ..."
cargo install --path .

# 2. Set up global configuration
CONFIG_DIR="$HOME/.config/meow"
THEMES_DIR="$CONFIG_DIR/themes"

echo "Setting up configuration in $CONFIG_DIR..."
mkdir -p "$THEMES_DIR"

# Copy main config if it doesn't exist
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    if [ -f ".config/config.toml" ]; then
        cp ".config/config.toml" "$CONFIG_DIR/config.toml"
        echo "Copied config.toml to $CONFIG_DIR"
    else
        echo "Warning: .config/config.toml not found, skipping copy."
    fi
else
    echo "Config file already exists in $CONFIG_DIR, skipping overwrite."
fi

# Copy themes
if [ -d ".config/themes" ]; then
    cp .config/themes/*.toml "$THEMES_DIR/"
    echo "Copied themes to $THEMES_DIR"
else
     echo "Warning: .config/themes directory not found."
fi

# 3. Check if ~/.cargo/bin is in PATH
CARGO_BIN="$HOME/.cargo/bin"

if [[ ":$PATH:" != *":$CARGO_BIN:"* ]]; then
    echo ""
    echo "WARNING: $CARGO_BIN is not in your PATH."
    echo "This means you won't be able to run 'meow' directly after closing this terminal."
    
    # Detect shell
    SHELL_NAME=$(basename "$SHELL")
    CONFIG_FILE=""

    if [ "$SHELL_NAME" = "zsh" ]; then
        CONFIG_FILE="$HOME/.zshrc"
    elif [ "$SHELL_NAME" = "bash" ]; then
        if [ -f "$HOME/.bashrc" ]; then
            CONFIG_FILE="$HOME/.bashrc"
        elif [ -f "$HOME/.bash_profile" ]; then
            CONFIG_FILE="$HOME/.bash_profile"
        fi
    fi

    if [ -n "$CONFIG_FILE" ]; then
        echo "Detected shell: $SHELL_NAME"
        echo "Configuration file: $CONFIG_FILE"
        echo ""
        read -p "Do you want me to add $CARGO_BIN to your PATH in $CONFIG_FILE? (y/n) " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "" >> "$CONFIG_FILE"
            echo "# Added by Meow Text Editor installer" >> "$CONFIG_FILE"
            echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\"" >> "$CONFIG_FILE"
            echo "Successfully updated $CONFIG_FILE."
            echo "Please restart your terminal or run 'source $CONFIG_FILE' to use 'meow'."
        else
            echo "Skipping PATH update."
            echo "You can manually add the following line to your shell config:"
            echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\""
        fi
    else
        echo "Could not detect your shell configuration file."
        echo "Please add the following line to your shell configuration (e.g., .zshrc, .bashrc):"
        echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    fi
else
    echo ""
    echo "Success! ~/.cargo/bin is already in your PATH."
    echo "You can now run 'meow' from anywhere."
fi

echo ""
echo "Installation complete!"
