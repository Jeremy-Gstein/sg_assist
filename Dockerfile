FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy built binary from host machine
COPY bin/sg_assistant /usr/local/bin/

# Non-root user for security
RUN useradd -m sg-admin
USER sg-admin 

# Run application binary as sg-admin user
CMD ["sg_assistant"]

