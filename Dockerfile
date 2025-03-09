FROM rust:latest AS builder
WORKDIR /app
COPY ./ ./
RUN cargo build --release

FROM linuxserver/ffmpeg:latest
COPY --from=builder /app/target/release/ffwatch /usr/local/bin/ffwatch
ENTRYPOINT ["ffwatch"]
