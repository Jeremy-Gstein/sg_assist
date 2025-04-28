# Stage 1: Includes all build utils
FROM rust:1-slim-bookworm AS builder
WORKDIR /sg_assistant
COPY . .
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

# Stage 2: Runtime image
FROM debian:bookworm-slim
WORKDIR /sg_assistant

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from stage 1 builder
COPY --from=builder /sg_assistant/target/release/sg_assistant /usr/local/bin
RUN useradd -m sg-admin
USER sg-admin
CMD ["sg_assistant"]

# Local Build:
# usage:
# docker run --env-file .env -itd --rm -v $PWD:/usr/src/sg_assistant -w /usr/src/sg_assistant sg_assist:latest 
# docker attach --detach-keys="ctrl-c" 

# RUN cargo install cargo-watch

# CMD ["cargo", "watch", "-w", "src", "-x", "run"]

