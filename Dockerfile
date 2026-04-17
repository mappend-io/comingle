FROM rust:1.93-trixie AS builder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update \
 && apt-get install -y musl-tools curl ca-certificates unzip \
 && rm -rf /var/lib/apt/lists/*

ENV BUN_INSTALL="/usr/local"
RUN curl -fsSL https://bun.sh/install | bash

WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl -p comingle

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/comingle /usr/bin/comingle
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
EXPOSE 3200
CMD ["/usr/bin/comingle"]
