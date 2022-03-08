FROM rust:latest as builder

RUN git clone https://github.com/FlorianKohn/RustOverflow.git
WORKDIR RustOverflow
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y sqlite3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/rustoverflow /usr/local/bin/rustoverflow
COPY --from=builder /RustOverflow/scss /RustOverflow/scss
COPY --from=builder /RustOverflow/static /RustOverflow/static
COPY --from=builder /RustOverflow/templates /RustOverflow/templates
COPY --from=builder /RustOverflow/Rocket.toml /RustOverflow/Rocket.toml
WORKDIR /RustOverflow
CMD ["rustoverflow"]