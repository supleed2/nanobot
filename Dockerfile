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
RUN apt-get update \
	&& apt-get install -y --no-install-recommends wget ca-certificates \
	&& rm -rf /var/lib/apt/lists/* && \
	groupadd --gid 1000 appgroup && \
	useradd --uid 1000 --gid appgroup --shell /usr/sbin/nologin appuser
COPY --chmod=644 fuzzy_linux_*.so ./
COPY --from=builder --chmod=755 /app/target/release/nano ./
ENV LD_LIBRARY_PATH=/app
EXPOSE 6266
HEALTHCHECK CMD wget --no-verbose --spider --tries=1 http://127.0.0.1:6266/up || exit 1
USER appuser
ENTRYPOINT ["./nano"]
