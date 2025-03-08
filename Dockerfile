FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y tree git && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/auto-pr /usr/local/bin/
COPY --from=builder /app/pr_template.hbs /usr/local/bin/

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]