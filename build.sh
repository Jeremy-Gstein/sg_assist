#!/usr/bin/bash

# Update code
git fetch && git pull

# Build Rust binary 
# Depends on default rustup toolchain
# Example: `rustup toolchain list`
#cargo build --release

# Extract tokens
# Assumes .env has DISCORD_TOKEN=... and WOWAUDIT_TOKEN=...
source .env 
# Docker rebuild
docker build -t shodo/sg_assist:main .

# Container management

# Ignore error if container doesn't exist
docker rm -f sg_app 2>/dev/null  
# Start sg_app
docker run --name sg_app -itd --restart always \
  -e DISCORD_TOKEN="$DISCORD_TOKEN" \
  -e WOWAUDIT_TOKEN="$WOWAUDIT_TOKEN" \
  shodo/sg_assist:main

# Watch initial logs
timeout --foreground 30 docker logs -f sg_app
