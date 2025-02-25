#!/bin/sh

set -ex
echo "[*] Starting tunnel..."
cloudflared tunnel --config /etc/cloudflared/config.yml run yss-ecs
