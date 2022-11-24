FROM rust:1.65.0 AS builder

WORKDIR /app
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM rust:1.65-slim-bullseye

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/* \

WORKDIR /app
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT prod
ENTRYPOINT ["./zero2prod"]