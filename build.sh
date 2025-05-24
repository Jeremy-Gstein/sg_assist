#!/usr/bin/bash

# Update code
sync_repo() {
  git fetch && git pull
}


# Stage 1:
# Build management
build() {
  # build and compile with cargo
  cargo build --release

  # build dockerfile
  docker build \
    --tag shodo/sg_assist:main \
    .
}

# Stage 2:
# Container management
start_container() {
  # Ignore error if container doesn't exist
  docker rm -f sg_app 2>/dev/null  
  # Assumes .env has DISCORD_TOKEN=... and WOWAUDIT_TOKEN=...
  source .env 
  # Start sg_app
  docker run --name sg_app -itd --restart always \
    -e DISCORD_TOKEN="$DISCORD_TOKEN" \
    -e WOWAUDIT_TOKEN="$WOWAUDIT_TOKEN" \
    -e WOWAUDIT_TOKEN_1="$WOWAUDIT_TOKEN_1" \
    -e MAINS="$MAINS" \
    -e ALTS="$ALTS" \
    -e RAIDER_ALTS="$RAIDER_ALTS" \
    -e OFFICER_ALTS="$OFFICER_ALTS" \
    shodo/sg_assist:main
}

# Watch initial logs
watch_docker_start() {
  timeout --foreground 30 watch -n 1 docker ps
}

# Development Workflow
start_dev() {
  build
  # Ignore error if container doesn't exist
  docker rm -f sg_app 2>/dev/null  
  # Assumes .env has DISCORD_TOKEN=... and WOWAUDIT_TOKEN=...
  source .env 
  # Start sg_app
  docker run --name sg_app -itd --restart always \
    -e DISCORD_TOKEN="$DISCORD_TOKEN" \
    -e WOWAUDIT_TOKEN="$WOWAUDIT_TOKEN" \
    -e WOWAUDIT_TOKEN_1="$WOWAUDIT_TOKEN_1" \
    -e MAINS="$MAINS" \
    -e ALTS="$ALTS" \
    -e RAIDER_ALTS="$RAIDER_ALTS" \
    -e OFFICER_ALTS="$OFFICER_ALTS" \
    -e RUST_LOG=debug \
    shodo/sg_assist:main
  clear
  echo "Attaching to container. Use ctrl-c to exit..."
  docker attach --detach-keys="ctrl-c" sg_app
}


if [ "$1" == "--dev" ]; then
  echo "Start Development Workflow"
  start_dev
else
  echo "Starting Default Build"
  build
  start_container
  watch_docker_start
fi
