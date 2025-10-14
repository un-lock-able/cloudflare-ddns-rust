#!/usr/bin/env bash
set -euo pipefail

BINARY_NAME="cloudflare-ddns-rust"
INSTALL_PATH="/usr/local/bin"
SYSTEMD_PATH="/etc/systemd/system"

echo "Building $BINARY_NAME..."
cargo build --release

echo "Installing binary to $INSTALL_PATH"
sudo install -m 755 "target/release/$BINARY_NAME" "$INSTALL_PATH/$BINARY_NAME"

read -p "Do you want to install systemd service and timer files? [y/N] " yn
if [[ "$yn" == [yY]* ]]; then
    echo "Installing systemd unit files to $SYSTEMD_PATH"
    sudo install -m 644 "systemd/$BINARY_NAME@.service" "$SYSTEMD_PATH/$BINARY_NAME@.service"
    sudo install -m 644 "systemd/$BINARY_NAME@.timer" "$SYSTEMD_PATH/$BINARY_NAME@.timer"

    echo "Reloading systemd daemon..."
    sudo systemctl daemon-reload

    # --- Step 4: Ask to enable timer for current user ---
    USER_NAME="$(whoami)"
    read -p "Do you want to enable timer for user '$USER_NAME'? [y/N] " yn2
    if [[ "$yn2" == [yY]* ]]; then
        sudo systemctl enable "$BINARY_NAME@$USER_NAME.timer"
        sudo systemctl start --now "$BINARY_NAME@$USER_NAME.timer"
        echo "Timer enabled for $USER_NAME"
        systemctl list-timers | grep "$BINARY_NAME@$USER_NAME.timer" || true
    else
        echo "Skipping timer enablement."
    fi
else
    echo "Skipping systemd unit installation."
fi

echo "Installation completed."