#!/bin/bash

set -e

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

sudo apt install libfontconfig-dev libjack-dev qjackctl

echo "@audio          -       rtprio          99" >> /etc/security/limits.conf