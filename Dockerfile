FROM rust:latest AS base

WORKDIR /app
COPY . .


FROM base AS build

RUN cargo build

ENTRYPOINT ["/app/target/debug/spire-game-server"]