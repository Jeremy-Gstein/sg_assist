#!/usr/bin/bash

# Enable Docker BuildKit
export DOCKER_BUILDKIT=1
CACHE_DIR="./target/docker-cache"
mkdir -p "$CACHE_DIR"

# Update code
git fetch && git pull
# build with cached assets
# see Dockerfile: ./target/docker-cache
# run `cargo clean` to clear cache 
docker build \
  --tag shodo/sg_assist:main \
  --build-arg BUILDKIT_INLINE_CACHE=1 \
  --cache-from type=local,src="$CACHE_DIR" \
  --cache-to type=local,dest="$CACHE_DIR, mode=max" \
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
