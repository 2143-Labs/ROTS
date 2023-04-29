FROM rust:latest

WORKDIR /usr/src/rots
COPY . .

RUN rustup install nightly
ENV CARGO_UNSTABLE_SPARSE_REGISTRY true
RUN cargo +nightly install --path server

CMD ["server"]
