# ------------------------------------------------------------------------------
# Frontend Build Stage
# ------------------------------------------------------------------------------

FROM node:16-alpine3.14 as frontend

WORKDIR /home/stump

COPY packages/core/frontend/ .

RUN npm install -g pnpm

RUN pnpm install
RUN pnpm run build

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:1-alpine3.15 as builder

ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN apk add --no-cache --verbose musl-dev build-base sqlite openssl-dev

WORKDIR /home/stump

COPY packages/core/server/ .

RUN cargo build --release --target=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

RUN apk add --no-cache libstdc++

RUN addgroup -g 1000 stump

RUN adduser -D -s /bin/sh -u 1000 -G stump stump

WORKDIR /home/stump

# copy the binary
COPY --from=builder /home/stump/target/x86_64-unknown-linux-musl/release/stump .

# copy the react build
COPY --from=frontend /home/stump/build static

COPY server/Rocket.toml .

RUN chown stump:stump stump

USER stump

ENV STUMP_CONFIG_DIR=/config
ENV STUMP_DATA_DIR=/data
ENV ROCKET_PROFILE=release

CMD ["./stump"]
