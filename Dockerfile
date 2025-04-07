FROM rust:latest
WORKDIR /usr/src/sg_assistant
COPY . .
#RUN cargo build --release
RUN cargo install cargo-watch
# CMD ["./target/release/sg_assistant"]
CMD ["cargo", "watch", "-w", "src", "-x", "run"]
