FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -m sg-admin

COPY target/release/sg_assistant /usr/local/bin/sg_assistant

USER sg-admin

CMD ["sg_assistant"]
