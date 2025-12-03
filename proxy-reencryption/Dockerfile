# Build stage
FROM rust:1.87-alpine AS build

ARG APP_NAME=proxy_reencryption

WORKDIR /build

RUN apk add openssl-dev musl-dev

COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "// dummy file" > src/lib.rs \
    && cargo build --release

COPY src src
RUN cargo build --locked --release
RUN cp ./target/release/$APP_NAME /bin/server

# Runtime stage
FROM debian:bullseye-slim AS final

COPY --from=build /bin/server /bin/

CMD ["/bin/server"]
