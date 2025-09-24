FROM --platform=linux/amd64 clux/muslrust:stable AS builder

WORKDIR /app
COPY . .

RUN unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy \
  && apt-get update && apt-get install -y pkg-config libssl-dev

RUN unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy \
  && cargo build --release --target x86_64-unknown-linux-musl

FROM --platform=linux/amd64 alpine:latest

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/lazy-mmdb .

CMD ["./lazy-mmdb"]
