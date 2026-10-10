#!/usr/bin/env bash

# Script for allowing the current $USER of the Linux OS to communicate with the ESP32 over Serial.
# Essential for flashing programs to the ESP32

# Tells script to exit immediately upon a pipeline failure
set -e

# Downloads official hardware rules and saves them to udev on your system
echo "Downloading Espressif udev rules..."
curl -fsSL https://raw.githubusercontent.com/espressif/openocd-esp32/master/contrib/60-openocd.rules | sudo tee /etc/udev/rules.d/60-openocd.rules > /dev/null

# Assigns $USER to group responsible for data transferring
# grep -q => 'quiet', does not print to stdout. Used to check if line exists.
if grep -q "^dialout:" /etc/group; then
    echo "Adding $USER to 'dialout' group (Debian/Ubuntu/Fedora)..."
    sudo usermod -a -G dialout $USER
    GROUP="dialout"
elif grep -q "^uucp:" /etc/group; then
    echo "Adding $USER to 'uucp' group (Arch Linux)..."
    sudo usermod -a -G uucp $USER
    GROUP="uucp"
else
    echo "Warning: Could not determine serial group. Manual setup required."
    exit 1
fi

# Apply rules immediately without restarting PC
echo "Reloading udev rules..."
sudo udevadm control --reload-rules
sudo udevadm trigger

echo "Serial setup complete! Run 'newgrp $GROUP' or log out/in of PC to apply."
