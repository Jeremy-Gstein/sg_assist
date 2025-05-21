#!/usr/bin/bash

build_image() {
  if [ -e ./Dockerfile ]; then
    docker build -t rustcc .
  else
    echo "no dockerfile found"
    exit 1
  fi
}

run_image() {
  # build for linux (static)
  docker run -d --rm --name rust-package -p 8080:8080 rustcc:latest
}

cleanup() {
 docker rm -f rust-package
}

create_package() {
  rm -rf package/
  mkdir package/
  check_env
}

get_package() {
  create_package
  curl -s "localhost:8080/static-package.tar" --output static-package.tar
}

build_static_linux() {
  build_image
  run_image
  sleep 2
  get_package
  sleep 2
  cleanup
}

build_linux() {
  # build for linux (dynamic)
  cargo build --release --target=x86_64-unknown-linux-gnu
  create_package
  cp -v ./target/x86_64-unknown-linux-gnu/release/debug-keysdone ./package
  tar czvf linux-package.tar ./package/
}

check_env() {
  if [ -e ./package/.env ]; then
    return 0
  elif [ -e ./.env ]; then
    cp -v ./.env ./package/.env
    return 0
  else
    echo ".env not found, must contain API keys" 
    exit 1
  fi
}

build_windows() {
  # build for windows 
  cargo build --release --target=x86_64-pc-windows-gnu 
  create_package
  cp -v ./target/x86_64-pc-windows-gnu/release/debug-keysdone.exe ./package
  tar czvf windows-package.tar ./package/
}

move_tar_to_package() {
  create_package
  mv -v *.tar ./package/
}


help_menu() {
  echo "Usage: $0 [--window|-w] [--linux|-l] [--static|-s] [--help|-h]"
  echo "  --windows, -w   Build for Windows"
  echo "  --linux,   -l   Build for Linux (dynamic link)"
  echo "  --static,  -s   Build for Linux (static link)"
  echo "  --help,    -h   Show this help menu" 
  # echo "Examples:"
  # echo "Build for 1 target by passing 1 arg"
  # echo "  ./build.sh --windows"
  # echo "Build for multiple targets by passing both args"
  # echo "  ./build.sh --windows --linux"
}

if [ $# -eq 0 ]; then
  help_menu
  exit 0
fi

while [[ $# -gt 0 ]]; do 
  case "$1" in 
    --windows|-w) 
      echo "Building for windows"
      build_windows
      ;;
    --linux|-l)
      echo "Building for linux (dynamic link)"
      build_linux
      ;;
    --static|-s)
      echo "Building for linux (static link)"
      build_static_linux
      ;;
    --help|-h)
      help_menu
      ;;
    *)
      echo "Invalid Input: $1"
      help_menu
      ;;
  esac
  shift
done
move_tar_to_package
