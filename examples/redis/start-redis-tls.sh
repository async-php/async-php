#!/bin/bash
# Start Redis with TLS support

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Starting Redis with TLS on port 6380..."
echo "Password: tls123"
echo ""

podman run --rm -it \
  -p 6380:6380 \
  -v "$SCRIPT_DIR:/tls:Z" \
  redis:alpine \
  redis-server /tls/redis-tls.conf
