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

### Updated 05/23/2025:
Instruction are work in progress, use build.sh for outdated build.
```shell
# build update_roster to generate list of main(nickname)/alts 
cd update_roster && cargo b --release
```
```shell
# build store_leaderboard to get mythic plus scores and append to redis db.
cd store_leaderboard && cargo b --release
```
Run the app with redis backend and poise/serenity backend for discord bot.
> [!NOTE]
> make sure to add your API key values to .env first or the build will fail.
```shell
docker compose up -d --build
```

- [invite to server](https://discord.com/oauth2/authorize?client_id=1274908402203627602)
