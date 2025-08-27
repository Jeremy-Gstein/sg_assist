#!/usr/bin/bash

# Requires Docker and Rust/cargo
# install Rust/cargo:
#   * curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

DEFAULT_PATH=/home/$USER/sg_assist
git_stuff() {
    echo "Fetching Latest Version"
    git fetch && git status
}

build_main() {
  # build main (serenity/poise discord backend)
  echo "building Discord Backend"
  cargo build --release
  cp -v target/release/sg_assistant $DEFAULT_PATH/bin/ && \
    cd $DEFAULT_PATH

}

build_leaderboard_cli_tool() {
  # build store_leaderboard (db cli tool)
  echo "building Leaderboard db tool"
  cd store_leaderboard && \
    cargo build --release && \
    cp -v target/release/keys-db $DEFAULT_PATH/bin/ && \
    cd $DEFAULT_PATH
}

build_roster_tool() {
  # build update_roster
  echo "building update_roster"
  cd update_roster && \
    cargo build --release && \
    cp -v target/release/keysdone $DEFAULT_PATH/bin/ && \
    cd $DEFAULT_PATH
}

start_services() {
  # start/enable systemd services.
  echo "moving systemd services to /etc/systemd/system/"
  cd $DEFAULT_PATH/store_leaderboard/services && \
    sudo cp -v delete-leaderboard.timer delete-leaderboard.service /etc/systemd/system/ && \
    sudo cp -v store_leaderboard.timer store_leaderboard.service /etc/systemd/system/ && \
    sudo systemctl daemon-reload && \
    sudo systemctl enable --now delete-leaderboard.timer && \
    sudo systemctl enable --now store_leaderboard.timer && \
    echo "Finished setting up systemd services.. check with:" && \
    echo "systemctl list-timers --all" && \
    cd $DEFAULT_PATH
}

compose_app_and_db() {
  echo "Starting Discord Bot and backend redis DB..."
  docker compose up -d --build
}

rebuild() {
  git_stuff
  build_main
  build_leaderboard_cli_tool
  build_roster_tool
}

####################
#  Update/Rebuild  #
####################
rebuild
####################
#  Docker/Runtime  #
####################
compose_app_and_db
####################
# Systemd Sercices #
####################
#start_services
