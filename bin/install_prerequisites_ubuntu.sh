#!/usr/bin/env bash
set -Eeuo pipefail

sudo apt-get update
sudo DEBIAN_FRONTEND=noninteractive apt-get -y install \
    curl
