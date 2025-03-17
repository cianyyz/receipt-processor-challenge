FROM rust:1.85.0 as builder

WORKDIR /usr/src/receipt-processor
COPY . .

RUN cargo build --release

FROM rust:1.85.0

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/receipt-processor/target/release/receipt-processor .

EXPOSE 8080

CMD ["receipt-processor"] 