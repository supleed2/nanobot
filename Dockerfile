FROM rust:trixie AS chef
WORKDIR /app
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json ./
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:trixie-slim AS app
WORKDIR /app
COPY --from=builder /app/target/release/nano ./
RUN apt-get update && apt-get install -y wget && rm -rf /var/lib/apt/lists/*
RUN groupadd -g 1000 appgroup && useradd -g appgroup -s /sbin/nologin -u 1000 appuser
RUN chown appuser:appgroup ./nano
EXPOSE 6266
HEALTHCHECK CMD wget --no-verbose --spider --tries=1 http://localhost:6266/up || exit 1
USER appuser
ENTRYPOINT ["./nano"]
