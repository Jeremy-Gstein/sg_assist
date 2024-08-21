# Seems Good Discord Assistant
[![Build](https://github.com/Jeremy-Gstein/sg_assist/actions/workflows/build.yml/badge.svg)](https://github.com/Jeremy-Gstein/sg_assist/actions/workflows/build.yml) [![Docker - amd64](https://img.shields.io/badge/Docker-amd64-2ea44f?logo=docker)](https://hub.docker.com/layers/shodo/sg_assist/main/images/sha256-6b01ea7953a5b1d648e2c0becbada4006dc336859582fac39d195d6b2c1b2342?context=repo) [![Docker - armv7](https://img.shields.io/badge/Docker-armv7-2ea44f?logo=docker)](https://hub.docker.com/layers/shodo/sg_assist/main/images/sha256-6b01ea7953a5b1d648e2c0becbada4006dc336859582fac39d195d6b2c1b2342?context=repo)

Discord bot to assist with repetitive discord actions.

- [invite to server](https://discord.com/oauth2/authorize?client_id=1274908402203627602)

---

## Build the app locally: 
- clone the repo
- make sure `$PWD` == `$PWD/sg_assistant`

Build the Docker image:
 ```shell
 docker build -t sg_assist .
```

Run and Start the Docker image:
```shell
docker run -it --name sg_assist_app sg_assist:latest
```

Or pull the image from dockerhub:
```shell
docker pull shodo/sg_assist
```

