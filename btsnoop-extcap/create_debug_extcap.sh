#! /usr/bin/env bash

# Run this script to generate a debug version of the extcap executable, which enables debug
# logging and writes them to /tmp/btsnoop-extcap.log.

set -ex

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# NOTE: Change to ~/.config/wireshark/extcap if using Wireshark 4.0 or older
WIRESHARK_PLUGIN_DEST="$HOME/.local/lib/wireshark/extcap/btsnoop-extcap"

if [[ -e "$WIRESHARK_PLUGIN_DEST" ]]; then
    echo "$WIRESHARK_PLUGIN_DEST already exists." >&2  
    read -p "Overwrite? [y/N] " response
    if [[ $response != "y" ]]; then
        exit 1
    fi
    rm "$WIRESHARK_PLUGIN_DEST"
fi

ln -s "$SCRIPT_DIR/../target/debug/btsnoop-extcap" "$WIRESHARK_PLUGIN_DEST"
