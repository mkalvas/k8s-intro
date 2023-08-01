FROM rust:slim as builder
COPY . /app
WORKDIR /app
RUN apt-get update && apt-get install pkg-config libssl-dev -y
RUN cargo build --release

# Strip final image to bare necessities
FROM gcr.io/distroless/cc
ENV DB_HOST="foundation-db-1"
COPY --from=builder /app/target/release/db-api /

CMD ["./db-api"]
