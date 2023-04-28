FROM rust:latest

WORKDIR /usr/src/rots
COPY . .

RUN rustup install nightly
ENV CARGO_UNSTABLE_SPARSE_REGISTRY true
RUN cargo +nightly install -Z no-index-update --path server

CMD ["server"]
