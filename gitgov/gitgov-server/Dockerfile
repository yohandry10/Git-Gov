FROM rust:1.88-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system gitgov \
    && useradd --system --gid gitgov --create-home --home-dir /home/gitgov --shell /usr/sbin/nologin gitgov

WORKDIR /app

COPY --from=builder /app/target/release/gitgov-server /usr/local/bin/gitgov-server

RUN chown -R gitgov:gitgov /app /home/gitgov

USER gitgov

EXPOSE 3000

ENV GITGOV_SERVER_ADDR=0.0.0.0:3000

CMD ["gitgov-server"]
