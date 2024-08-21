FROM rust:latest
WORKDIR /usr/src/sg_assistant
COPY . .
RUN cargo build --release
CMD ["./target/release/sg_assistant"]
