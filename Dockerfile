# Stage 1: Planner
FROM rust:1-bookworm AS planner
WORKDIR /sg_assistant
RUN cargo install cargo-chef --locked
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder
FROM rust:1-bookworm AS builder
WORKDIR /sg_assistant
RUN cargo install cargo-chef --locked
COPY --from=planner /sg_assistant/recipe.json .
# Get build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Build dependencies
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
WORKDIR /sg_assistant
RUN apt-get update && \
    apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /sg_assistant/target/release/sg_assistant /usr/local/bin
RUN useradd -m sg-admin
USER sg-admin
CMD ["sg_assistant"]
