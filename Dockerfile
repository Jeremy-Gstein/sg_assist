FROM rust:latest
WORKDIR /usr/src/sg_assistant
COPY . .
RUN cargo build --release
CMD ["./target/release/sg_assistant"]


# Local Build:
# usage:
# docker run --env-file .env -itd --rm -v $PWD:/usr/src/sg_assistant -w /usr/src/sg_assistant sg_assist:latest 
# docker attach --detach-keys="ctrl-c" 

# RUN cargo install cargo-watch

# CMD ["cargo", "watch", "-w", "src", "-x", "run"]

