FROM rust:1 AS chef 
RUN cargo install cargo-chef 
RUN adduser rots
WORKDIR /rots

FROM chef AS planner
WORKDIR /rots
COPY --chown=rots:rots . .
RUN cargo chef prepare --bin server --recipe-path recipe.json
USER rots

FROM debian:12-slim AS base
RUN apt-get update
RUN apt-get install -y bash

FROM chef AS builder
WORKDIR /rots
COPY --from=planner /rots/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY --chown=rots:rots . .
WORKDIR /rots/server
RUN cargo build --release --bin server

# We do not need the Rust toolchain to run the binary!
FROM base AS runner

RUN mkdir -p /rots
WORKDIR /rots
RUN adduser rots


COPY --from=builder --chown=rots:rots /rots/target/release/server /rots/server
CMD ["bash", "-c", "echo starting server; /rots/server"]
