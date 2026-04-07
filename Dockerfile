FROM rust:1.93-trixie AS builder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools curl ca-certificates \
 && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
 && apt-get install -y nodejs \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl -p comingle

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/comingle /usr/bin/comingle
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
EXPOSE 3200
CMD ["/usr/bin/comingle"]
