FROM rust:1.88-bookworm AS builder

WORKDIR /app

COPY gitgov-server/Cargo.toml gitgov-server/Cargo.lock ./gitgov-server/
COPY policy-core/Cargo.toml ./policy-core/
COPY policy-core/src ./policy-core/src
COPY gitgov-server/src ./gitgov-server/src

RUN cargo build --release --manifest-path gitgov-server/Cargo.toml

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system gitgov \
    && useradd --system --gid gitgov --create-home --home-dir /home/gitgov --shell /usr/sbin/nologin gitgov

WORKDIR /app

COPY --from=builder /app/gitgov-server/target/release/gitgov-server /usr/local/bin/gitgov-server

RUN chown -R gitgov:gitgov /app /home/gitgov

USER gitgov

EXPOSE 3000

ENV GITGOV_SERVER_ADDR=0.0.0.0:3000

CMD ["gitgov-server"]
