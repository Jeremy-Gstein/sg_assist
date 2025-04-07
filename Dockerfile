FROM rust:latest
WORKDIR /usr/src/sg_assistant
COPY . .
RUN cargo build --release
CMD ["./target/release/sg_assistant"]


# Local Build
# add comment to RUN/CMD above
# RUN cargo install cargo-watch

# CMD ["cargo", "watch", "-w", "src", "-x", "run"]

