#!/usr/bin/bash

# Update code
git fetch && git pull

# Build management

cargo build --release

# build dockerfile
docker build \
  --tag shodo/sg_assist:main \
  .



# Container management

# Ignore error if container doesn't exist
docker rm -f sg_app 2>/dev/null  
# Assumes .env has DISCORD_TOKEN=... and WOWAUDIT_TOKEN=...
source .env 
# Start sg_app
docker run --name sg_app -itd --restart always \
  -e DISCORD_TOKEN="$DISCORD_TOKEN" \
  -e WOWAUDIT_TOKEN="$WOWAUDIT_TOKEN" \
  shodo/sg_assist:main

# Watch initial logs
timeout --foreground 30 watch -n 1 docker ps
