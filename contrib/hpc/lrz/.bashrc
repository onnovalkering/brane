#!/usr/bin/env bash
module load nano
module load charliecloud/0.11

# Create local bin directory
USER_BIN_DIR="$HOME/.local/bin"
if [[ ! -d $USER_BIN_DIR ]]; then
    mkdir -p $USER_BIN_DIR
fi

PATH="$USER_BIN_DIR:$PATH"

# Install Chaplin, if not already present.
if [[ ! -x "$(command -v chaplin)" ]]; then
    curl -L https://git.io/JJLps -o chaplin
    chmod +x chaplin
    mv chaplin "$USER_BIN_DIR/chaplin"
fi
