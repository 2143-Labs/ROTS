FROM rust:latest

WORKDIR /usr/src/rots
COPY . .

RUN cargo install --path server

CMD ["server"]
