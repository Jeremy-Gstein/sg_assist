FROM alpine:latest AS builder

RUN apk add pkgconf openssl-dev curl binutils build-base musl-utils openssl-libs-static python3

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain nightly --profile minimal \
    && source "$HOME/.cargo/env" \
    && rustup target add x86_64-unknown-linux-musl
ENV PATH="/root/.cargo/bin:${PATH}"
    
WORKDIR /build
COPY ./Cargo.toml ./Cargo.lock ./
COPY src ./src/
COPY ./sg-alts.yaml ./.env ./
RUN cargo build --release

FROM builder AS package
RUN mkdir /build/package
RUN mv /build/target/release/debug-keysdone /build/package
RUN mv /build/.env /build/package
WORKDIR /build
RUN tar czvf static-package.tar ./package 
RUN mv static-package.tar ./package


FROM package AS server
WORKDIR /build/package
EXPOSE 8080
CMD ["python3", "-m", "http.server", "--bind", "0.0.0.0", "8080"]  
# docker run --rm -p 8080:8080 rustcc:latest
# curl -s localhost:8080/debug-keysdone --output debug-keysdone
