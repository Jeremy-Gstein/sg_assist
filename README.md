# Seems Good Discord Assistant

Discord bot to assist with repetitive discord actions.

---

# Build the app locally:
Choose to build and run with docker OR build and run with cargo
### Quickstart With `docker`:
Install dependencies (rust! 🦀)
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
clone the repo
```shell
git clone https://github.com/Jeremy-Gstein/sg_assist && cd sg_assist
```
create a .env file inside sg_assit/
- add your discord and wowaudit token to .env
```sh
touch .env
echo DISCORD_TOKEN=enter_your_token_here >> .env
echo WOWAUDIT_TOKEN=enter_your_token_here >> .env
```
make the build script executable and run it
```shell
chmod +x build.sh
./build.sh
```
---

### Quickstart With `cargo`:
Install dependencies (rust! 🦀)
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
clone the repo
```shell
git clone https://github.com/Jeremy-Gstein/sg_assist && cd sg_assist
```
export your discord and wowaudit token
```sh
export DISCORD_TOKEN=enter_your_token_here
export WOWAUDIT_TOKEN=enter_your_token_here
```
build with cargo 
```shell
cargo build --release
```
run with cargo 
```shell
cargo run --release
```
---
- [invite to server](https://discord.com/oauth2/authorize?client_id=1274908402203627602)
