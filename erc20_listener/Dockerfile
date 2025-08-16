FROM rust:1.88.0-slim as build-env
WORKDIR /app
COPY . /app

RUN apt-get update && apt-get install -y musl-dev gcc libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=build-env /app/target/release/erc20_listener /
CMD ["./erc20_listener"]